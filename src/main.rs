extern crate unicode_normalization;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::BufRead;
use unicode_normalization::UnicodeNormalization;

fn list_languages(path: &str) {
    let lines =
        std::io::BufReader::new(std::fs::File::open(path).expect("Could not open input file"))
            .lines();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut count = 0usize;

    for line in lines.map(|l| l.expect("Could not read a line in file")) {
        if let Some(lang) = line.split('\t').nth(1) {
            count += 1;
            if !seen.contains(lang) {
                seen.insert(lang.to_string());
                println!("{}", lang);
            }
        }
    }

    eprintln!("Found {} languages in {} sentences", seen.len(), count);
}

fn make_translations<F>(
    sentence_path: &str,
    link_path: &str,
    lang_from: &str,
    lang_to: &str,
    tokenize: F,
) where
    for<'a> F: Clone + FnMut(&'a str) -> Vec<&'a str>,
{
    eprintln!("Finding word frequencies for '{}'...", lang_from);
    let word_frequencys = word_frequency(sentence_path, lang_from, tokenize.clone());

    eprintln!("Sorting and indexing words...");
    let word_to_freq: BTreeMap<String, usize> = {
        let mut ifreq_to_word: BTreeMap<usize, Vec<String>> = BTreeMap::new();
        for (word, freq) in word_frequencys {
            let ifreq = std::usize::MAX - freq;
            if let Some(words) = ifreq_to_word.get_mut(&ifreq) {
                words.push(word);
            } else {
                ifreq_to_word.insert(ifreq, vec![word]);
            }
        }

        ifreq_to_word
            .into_iter()
            .enumerate()
            .flat_map(|(freq, (_, words))| words.into_iter().map(move |word| (word, freq)))
            .collect()
    };

    eprintln!("Ordering sentences by ease...");
    let from_sentences = get_sentence_scores(sentence_path, lang_from, &word_to_freq, tokenize);

    eprintln!("Reading sentence links...");
    let links = parse_links(sentence_path, link_path, lang_from, lang_to);

    eprintln!("Fetching required translations...");
    let translations = get_translations(sentence_path, &links);

    eprintln!("Outputting file...");
    let mut skipped = 0usize;

    for (id, sentence, _score) in from_sentences {
        if let Some(translation_id) = links.get(&id).and_then(|ls| ls.get(0)) {
            if let Some(translation) = translations.get(&translation_id) {
                println!("{}\t{}\t{}\t{}", id, translation_id, sentence, translation);
                continue;
            }
        }

        skipped += 1;
    }

    eprintln!("Could not find translations for {} sentences", skipped);
}

fn get_translations(
    sentence_path: &str,
    links: &BTreeMap<usize, Vec<usize>>,
) -> BTreeMap<usize, String> {
    let required: BTreeSet<usize> = links.iter().flat_map(|(_, vs)| vs).cloned().collect();

    let mut translations = BTreeMap::new();

    let lines = std::io::BufReader::new(
        std::fs::File::open(sentence_path).expect("Could not open input file"),
    )
    .lines();

    for line in lines.map(|l| l.expect("Could not read a line in file")) {
        let mut iter = line.split('\t');
        let id = iter
            .next()
            .map(|id| id.parse::<usize>().expect("Could not parse id"));
        if let Some(id) = id {
            if required.contains(&id) {
                if let (_, Some(sentence)) = (iter.next(), iter.next()) {
                    translations.insert(id, sentence.to_string());
                }
            }
        }
    }

    translations
}

fn parse_links(
    sentence_path: &str,
    link_path: &str,
    lang_from: &str,
    lang_to: &str,
) -> BTreeMap<usize, Vec<usize>> {
    let mut from_ids: BTreeSet<usize> = BTreeSet::new();
    let mut to_ids: BTreeSet<usize> = BTreeSet::new();

    {
        let lines = std::io::BufReader::new(
            std::fs::File::open(sentence_path).expect("Could not open input file"),
        )
        .lines();

        for line in lines.map(|l| l.expect("Could not read a line in file")) {
            let mut iter = line.split('\t');
            let id = iter
                .next()
                .map(|id| id.parse::<usize>().expect("Could not parse id"));
            match (id, iter.next()) {
                (Some(id), Some(lang)) if lang_from == lang => {
                    from_ids.insert(id);
                }
                (Some(id), Some(lang)) if lang_to == lang => {
                    to_ids.insert(id);
                }
                _ => {}
            }
        }
    }

    let mut links: BTreeMap<usize, Vec<usize>> = BTreeMap::new();

    {
        let lines = std::io::BufReader::new(
            std::fs::File::open(link_path).expect("Could not open input file"),
        )
        .lines();

        for line in lines.map(|l| l.expect("Could not read a line in file")) {
            let mut iter = line.split('\t');
            let id1 = iter
                .next()
                .map(|id| id.parse::<usize>().expect("Could not parse id"));
            let id2 = iter
                .next()
                .map(|id| id.parse::<usize>().expect("Could not parse id"));
            if let (Some(id1), Some(id2)) = (id1, id2) {
                if from_ids.contains(&id1) && to_ids.contains(&id2) {
                    if let Some(ls) = links.get_mut(&id1) {
                        ls.push(id2);
                    } else {
                        links.insert(id1, vec![id2]);
                    }
                }
            }
        }
    }

    links
}

fn get_sentence_scores<F>(
    sentence_path: &str,
    lang_from: &str,
    word_to_freq: &BTreeMap<String, usize>,
    mut tokenize: F,
) -> Vec<(usize, String, usize)>
where
    for<'a> F: Clone + FnMut(&'a str) -> Vec<&'a str>,
{
    let mut lines: Vec<(usize, String, usize)> = std::io::BufReader::new(
        std::fs::File::open(sentence_path).expect("Could not open input file"),
    )
    .lines()
    .map(|l| l.expect("Could not read a line in file"))
    .filter_map(|line| {
        let mut iter = line.split('\t');
        if let (Some(id), Some(lang), Some(sentence)) = (iter.next(), iter.next(), iter.next()) {
            if lang == lang_from {
                let scores = filtered_words(tokenize(sentence).into_iter())
                    .into_iter()
                    .map(|word| word_to_freq.get(&word).cloned())
                    .collect::<Option<Vec<usize>>>();

                scores
                    .and_then(|scores| scores.into_iter().max())
                    .map(|high_score| {
                        (
                            id.parse::<usize>().expect("Could not parse sentence id"),
                            sentence.to_string(),
                            high_score,
                        )
                    })
            } else {
                None
            }
        } else {
            None
        }
    })
    .collect();

    lines.sort_unstable_by_key(|l| l.2);
    lines
}

fn word_frequency<F>(sentence_path: &str, lang: &str, mut tokenize: F) -> BTreeMap<String, usize>
where
    for<'a> F: FnMut(&'a str) -> Vec<&'a str>,
{
    let lines = std::io::BufReader::new(
        std::fs::File::open(sentence_path).expect("Could not open input file"),
    )
    .lines();

    let mut seen: BTreeMap<String, usize> = BTreeMap::new();

    for line in lines.map(|l| l.expect("Could not read a line in file")) {
        let cells: Vec<&str> = line.split('\t').collect();

        if cells.get(1) == Some(&lang) {
            if let Some(sentence) = cells.get(2) {
                for word in filtered_words(tokenize(sentence).into_iter()) {
                    let count = seen.get(&word).unwrap_or(&0);
                    seen.insert(word, *count + 1);
                }
            }
        }
    }

    seen
}

fn filtered_words<'a>(tokens: impl Iterator<Item = &'a str>) -> Vec<String> {
    fn is_part_number(word: &str) -> bool {
        word.chars().any(char::is_numeric)
    }
    fn is_part_split_char(word: &str) -> bool {
        word.chars().any(is_split_char)
    }

    tokens
        .filter(|w| !is_part_number(w) && !is_part_split_char(w) && *w != "")
        .map(|word| word.nfc().collect::<String>().to_uppercase())
        .collect()
}

fn is_split_char(c: char) -> bool {
    let overrides = ['\''];

    !overrides.contains(&c) && (c.is_whitespace() || c.is_ascii_punctuation())
}

const USAGE: &str = "
Usage:
- To print all languages:
    tatoeba-frequency langs <sentences.csv path>
- To create translations to stdout:
    tatoeba-frequency ease <lang from> <lang to> <sentences.csv path> <links.csv path>";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match &args[..] {
        [_, cmd, path] if cmd == "langs" => list_languages(path),
        [_, cmd, from, to, sentence_path, link_path] if cmd == "ease" => {
            make_translations(sentence_path, link_path, from, to, |line| {
                line.split(is_split_char).collect()
            })
        }
        _ => {
            eprintln!("{}", USAGE);
            std::process::exit(1);
        }
    }
}

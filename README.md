# Create a list of translations from any two languages on tatoeba

- Creates translated sentences for studying
- Ordered by ease
    - Uses word frequency to calculate ease of sentence
    - Word frequency is calculated based on the sentences in tatoeba
    - A sentence is ranked as hard as its hardest word
- Currently only able to parse languages where words are separated by white-space
    - Work in progress for plug-able tokenizers to support Chinese, Japanese etc

## Requirements

- Requires tatoeba data dumps from here: https://tatoeba.org/eng/downloads
    - `sentences.csv`
    - `links.csv`

## Usage
```
- To print all languages:
    tatoeba-frequency langs <sentences.csv path>
- To create translations to stdout:
    tatoeba-frequency ease <lang from> <lang to> <sentences.csv path> <links.csv path>
```

## Output format

- Tab seperated file with the following columns
    - Source sentence ID
    - Translation ID
    - Source sentence content
    - Translation content

## Example French to English output

Easy sentences at the top of the output file:
```
506830	42344	Est-ce à vous ?	Is it yours?
2080656	1259560	Est-ce vous ?	Is it you?
6468622	1886899	Est-ce Tom ?	Is that Tom?
8973	16491	Et vous ?	How about you?
7801750	6586596	Pour vous ?	For you?
```

Hard/uncommon sentences near the end of the file:
```
7778149	2640472	Ce n'est pas une bonne idée que de prendre des auto-stoppeurs.	It's not a good idea to pick up hitchhikers.
134780	318318	Puis-je avoir un sac isotherme ?	May I have an ice bag?
134778	318283	N'emportez pas plus d'argent que nécessaire.	Don't carry more money than you need.
1225958	57407	Que devons-nous faire avec cette délinquante ?	What shall we do with this delinquent girl?
1226043	1225943	Cette vis est desserrée.	This screw is loose.
```

# Example English to Latin output

Easier
```
3382699	3382914	Mary! Mary! Mary!	Maria! Maria! O Maria!
2243298	6866380	They said that.	Id dixerunt.
5226837	5223611	He said that he was Tom.	Thoman se esse dixit.
5226832	5223605	She said that she was Mary.	Mariam se esse dixit.
3053024	6951623	Is this you?	Estne hic tu?
```

Harder
```
6379268	3576401	Delphi is also a small town.	Delphi quoque oppidum parvum est.
4256176	4227675	Vampires live in perpetuity.	Vampyri in perpetuum vivant.
7356342	7356333	A mousetrap rids the house of mice.	Muscipula domum a muribus purgat.
7356337	7356331	A cat rids the house of mice.	Felis domum a muribus purgat.
6303172	6303721	Late-comers were not admitted.	Tardi non admittebantur.
```

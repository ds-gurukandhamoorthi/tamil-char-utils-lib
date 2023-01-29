#![allow(
    clippy::manual_strip
)]

use unicode_segmentation::UnicodeSegmentation;
use cpython::{Python, PyResult, py_module_initializer, py_fn, PyDict, PyList, PyBool};
use regex::Regex;
use once_cell::sync::Lazy;

static EXCLUSIVELY_TAMIL_WORD_CHARACTERS : Lazy<Regex> = Lazy::new(||{
    // tamil letter (with an optional tamil mark)
    Regex::new(r"^([\p{Tamil}&&\pL][\p{Tamil}&&\pM]?)*$").unwrap()
});

fn is_vowel(c: &str) -> bool{
    "அஆஇஈஉஊஎஏஐஒஓஔஃ".contains(c)
}

fn is_consonant(c: &str) -> bool{
    c == "ஸ்ர" || c == "க்ஷ" || "கசடதபறயரலவழளஞஙனநமணஸஷஹஜ".contains(c)
}

fn is_mark(c: &str) -> bool{
    let marks = " ்ாிீுூெேைொோௌ";
    marks.contains(c)
}

fn is_marked_consonant(en: &str) -> bool {
    en.starts_with(|c| is_consonant(&format!("{c}"))) &&
    en.ends_with(|c| is_mark(&format!("{c}")))
    //FIXME: two marks riding the same entity is accepted as of now.
}


fn is_tamil_entity(e: &str) -> bool{
    is_vowel(e) || is_consonant(e) || is_marked_consonant(e)
}

//The two words in the arguments are supposed to be valid words, so we can forgo the verification
//above `is_tamil_entity`
fn dist_word(word1: &str, word2: &str) -> usize {
    let ents1 = word1.graphemes(true).collect::<Vec<&str>>();
    let ents2 = word2.graphemes(true).collect::<Vec<&str>>();
    strsim::generic_levenshtein(&ents1, &ents2)
}

// A word composed exclusively of tamil word characters
fn is_tamil_word(word: &str) -> bool {
    EXCLUSIVELY_TAMIL_WORD_CHARACTERS.is_match(word)
}

fn dist_word_py(_:Python, word1:&str, word2: &str) -> PyResult<u32> {
    Ok(dist_word(word1, word2) as u32)
}

//we return minimum distance
fn dist_word_to_wordlist_py(py:Python, word:&str, wordlist: &PyList) -> PyResult<u32> {
    let mut mindist = u32::max_value();
    for w in wordlist.iter(py) {
        let w = w.to_string();
        let dist = dist_word(word, &w) as u32;
        if  dist < mindist {
            mindist = dist
        }
    }
    Ok(mindist)
}

//This doesn't verify word is constructed of valid tamil entities. No need to. As we wouldn't
//have any equivalent key in rules-dict for those entities
fn unigram_auto(py:Python, word: &str, rules: &PyDict) -> PyResult<String> {
    let mut res = String::new();
    for ent in word.graphemes(true) {
        if rules.contains(py, ent).unwrap() {
            res.push_str(&format!("{}", rules.get_item(py, ent).unwrap()))
        } else {
            return Ok(String::from(""))
        }
    }
    Ok(res)
}

fn is_tamil_word_py(py:Python, word: &str) -> PyResult<PyBool> {
    let res = if is_tamil_word(word) {py.True()} else {py.False()};
    Ok(res)
}


//number of entities in a word. There shouldn't be any other thing like space, ascii etc..
//this functions verifies that the word is composed of valid tamil entities.
fn nb_valid_tamil_entities(_:Python, string: &str) -> PyResult<u32> {
    let grs = string.graphemes(true);
    let (count_grs, only_tamil_grs) = grs.fold((0, true), |acc, g| {
        let (count, tamil_only) = acc;
        if is_tamil_entity(g) {
            (count + 1, tamil_only)
        } else {
            (count, false)
        }
    });
    if only_tamil_grs {
        Ok(count_grs)
    }else {
        Ok(0)
    }
}

//No verification is done. We can use this for other languages too: cab -> abc
fn unique_sorted_entities(_:Python, word: &str) -> PyResult<String> {
    let mut ents = word.graphemes(true).collect::<Vec<&str>>();
    ents.sort_unstable();
    ents.dedup();
    Ok(ents.join(""))
}

py_module_initializer!(tamilcharutils, |py, m| {
    m.add(py, "__doc__", "Module written in Rust for tamil character utils")?;
    m.add(py, "nb_valid_tamil_entities", py_fn!(py, nb_valid_tamil_entities(string: &str)))?;
    m.add(py, "dist_word", py_fn!(py, dist_word_py(word1: &str, word2: &str)))?;
    m.add(py, "dist_word_to_wordlist", py_fn!(py, dist_word_to_wordlist_py(word1: &str, wordlist: &PyList)))?;
    m.add(py, "unigram_auto", py_fn!(py, unigram_auto(word: &str, rules: &PyDict)))?;
    m.add(py, "unique_sorted_entities", py_fn!(py, unique_sorted_entities(word: &str)))?;
    m.add(py, "is_tamil_word", py_fn!(py, is_tamil_word_py(word: &str)))?;
    Ok(())
});

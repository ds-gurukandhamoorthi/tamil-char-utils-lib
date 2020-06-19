use unicode_segmentation::UnicodeSegmentation;
use cpython::{Python, PyResult, py_module_initializer, py_fn};

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
    en.starts_with(|c| is_consonant(&format!("{}", c))) &&
    en.ends_with(|c| is_mark(&format!("{}", c)))
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

fn dist_word_py(_:Python, word1:&str, word2: &str) -> PyResult<u32> {
    Ok(dist_word(word1, word2) as u32)
}




//number of entities in a word. There shouldn't be any other thing like space, ascii etc..
//this functions verifies that the word is composed of valid tamil entities.
fn nb_valid_tamil_entities(_:Python, string: &str) -> PyResult<u32> {
    let mut count = 0_u32;
    for ent in string.graphemes(true) {
        match ent {
            e if is_tamil_entity(e) => {count += 1}
            _ => {return Ok(0)}
        }
    }
    Ok(count)
}

py_module_initializer!(tamilcharutils, |py, m| {
    m.add(py, "__doc__", "Module written in Rust for tamil character utils")?;
    m.add(py, "nb_valid_tamil_entities", py_fn!(py, nb_valid_tamil_entities(string: &str)))?;
    m.add(py, "dist_word", py_fn!(py, dist_word_py(word1: &str, word2: &str)))?;
    Ok(())
});

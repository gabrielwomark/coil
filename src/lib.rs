use std::collections::{hash_map::DefaultHasher,HashMap};
use std::f64;
use std::hash::{Hash, Hasher};

type TokId = u64;
type DocId = usize;

pub fn hash_str(s: &str) -> TokId {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}


pub fn tokenize(input: &str) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for tok in input.replace("\n", " ").split_whitespace().map(|t| t.to_lowercase()) {
        *counts.entry(tok).or_insert(0) += 1;
    }
    return counts
}

#[derive(Debug, Clone)]
pub struct ScoreVec {
    values: Vec<i32>,
    stride: usize,
    size: usize
}


impl ScoreVec {
    pub fn new(stride: usize) -> Self {
        Self{values:Vec::new(), stride, size: 0}

    }

    pub fn get(&self, index: usize) -> Result<&[i32], String>  {
        if index > (self.values.len() / self.stride) {
            return Err("Index out of bounds".to_string());
        }
        let start = self.stride * index;
        let end = start + self.stride;
        Ok(&self.values[start..end])
    }

    pub fn add(&mut self, score: &mut Vec<i32>) -> Result<(), String> {
        let to_add = self.size;
        if to_add > (self.values.len() / self.stride) {
            return Err("Index out of bounds".to_string());
        }

        if score.len() != self.stride {
            return Err(format!("size vector shape doesn't match, size: {0}, stride: {1}", score.len(), self.stride))
        }
        self.values.extend(score.drain(..));
        self.size += 1;
        Ok(())
        
    }
}


#[derive(Debug, Clone)]
pub struct HitList {
    doc_ids: Vec<usize>,
    scores: ScoreVec
}

impl HitList {
    pub fn new(score_dims: usize) -> Self {
        HitList{doc_ids:Vec::new(), scores:ScoreVec::new(score_dims)}
    }

    pub fn add_doc(&mut self, doc_id: usize, scores:&mut Vec<i32>) -> Result<(), String> {
        self.doc_ids.push(doc_id);
        self.scores.add(scores)?;
        Ok(())
    }

    pub fn get_docs(&self) -> &[usize] {
        &self.doc_ids
    }

    pub fn get_scores(&self) -> &ScoreVec {
        &self.scores
    }

}


#[derive(Debug)]
pub struct Index {
    inverted: HashMap<TokId, HitList>,
    forward: HashMap<DocId, String>,
    score_dims: usize,
    num_docs: usize,
}

impl Index {
    pub fn new(score_dims: usize) -> Self {
        Self{inverted: HashMap::new(), score_dims, forward: HashMap::new(), num_docs: 0}

    }

    pub fn num_docs(&self) -> &usize {
        return &self.num_docs
    }

    pub fn index(&mut self, tok_id: TokId, doc_id: DocId, scores: &mut Vec<i32>, title: String) -> Result<(), String> {
        let hitlist = self.inverted.entry(tok_id).or_insert_with(|| HitList::new(self.score_dims));
        hitlist.add_doc(doc_id, scores)?;
        self.forward.insert(doc_id, title);
        self.num_docs += 1;
        Ok(())
    }

    pub fn search(&self, query: &str, num_hits: usize) -> Vec<(String, f64)> {
        let tokens = tokenize(query);
        let mut scores: HashMap<DocId, f64> = HashMap::new();
        for token in tokens.keys() {
            let tok_id = hash_str(token);
            let hitlist = self.inverted.get(&tok_id);
            let hits = hitlist.map(|hits| {hits.get_docs()}).unwrap_or(&[]);

            for (doc_idx, doc_id) in hits.iter().enumerate() {
                let doc_score = scores.entry(*doc_id).or_insert(0.0);
                let term_frequency = hitlist.unwrap().get_scores().get(doc_idx).unwrap()[0];
                let doc_freq = hits.len() as f64;
                *doc_score += f64::ln ((self.num_docs as f64 -  doc_freq) / doc_freq) * term_frequency as f64;
            }
        }

        let mut items: Vec<(DocId, f64)> = scores.into_iter().collect();
        items.sort_by(|hita, hitb| hita.1.partial_cmp(&hitb.1).unwrap());
        items.into_iter().rev().take(num_hits).map(|(doc_id, score)| { (self.forward.get(&doc_id).unwrap().clone(), score) }).collect()
    }
}

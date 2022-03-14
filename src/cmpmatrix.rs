use std::collections::HashMap;

#[derive(Clone)]
pub struct FuncSig {
    pub module: String,
    pub symbol: String,
    pub sig: String,
    pub index: usize,
}

pub struct CmpMatrix {
    entries: Vec<FuncSig>,
    lookup: HashMap<String,usize>,
    compares: Vec<Vec<f32>>,
}

impl CmpMatrix {
    pub fn new() -> Self {
        Self {
            entries: vec![],        // Ordered vector of column & row entries
            lookup: HashMap::new(), // Lookup provides a quick "symbol" -> "index" match
            compares: vec![],
        }
    }

    pub fn add(self: &mut Self, m: String, s: String, sig: String) -> Option<()> {
        let k = m.clone() + ":" + s.as_str();
        match self.lookup.get(&k) {
            Some(_) => {
                None
            },
            None => {
                // Initialize the new element
                let b = FuncSig {
                    module: m.clone(),
                    symbol: s.clone(),
                    sig: sig.clone(),
                    index: self.entries.len()
                };

                // Insert it into the ordered storage array
                self.entries.push(b);

                // Store the new element's index in the HashMap that does lookup by symbol
                self.lookup.insert(k, self.entries.len() - 1);

                // Expand the dimensions of the comparison matrix, initializing to 0.0
                self.compares.iter_mut().for_each(|x| x.push(0.0));
                self.compares.push(vec![0.0; self.entries.len()]);
                Some(())
            }
        }
    }

    pub fn get_compare_val(self: &Self, left: usize, right: usize) -> f32 {
        self.compares[left][right]
    }

    pub fn get_entries(self: &Self) -> Vec<FuncSig> {
        self.entries.clone()
    }

    pub fn entries_len(self: &Self) -> usize {
        self.entries.len()
    }

    pub fn get_entry(self: &Self, index: usize) -> Result<FuncSig, &'static str> {
        Ok(self.entries[index].clone())
    }

    pub fn update_by_index(self: &mut Self, left: usize, right: usize, val: f32) -> Result<(), &'static str> {
        self.compares[left][right] = val;
        Ok(())
    }

    pub fn update(self: &mut Self, left: &String, right: &String, val: f32) -> Result<(), &'static str> {
        match self.lookup.get(left) {
            Some(&lindex) => {
                match self.lookup.get(right) {
                    Some(&rindex) => {
                        self.update_by_index(lindex, rindex, val)
                    },
                    None => {
                        Err("Right key not found")
                    }
                }
            },
            None => {
                Err("Left key not found")
            }
        }
    }
}

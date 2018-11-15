use std::fmt::{Display, Error, Formatter};

#[derive(Debug, Clone)]
pub struct Location {
    pieces: Vec<String>,
}

impl Location {
    pub fn empty( ) -> Self {
        Location::new(vec![])
    }
    pub fn new(pieces: Vec<&str>) -> Self {
        Location {
            pieces: pieces.clone().into_iter().map(|a| a.to_string()).collect(),
        }
    }

    pub fn add(&self, piece: &str) -> Self {
        let mut new_location = self.clone();
        new_location.pieces.push(piece.to_string());
        new_location
    }

    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let output = self.pieces.join("::");
        write!(f, "{}", output)
    }
}



#[derive(Debug, Clone)]
pub struct Disparity {
    message: String,
    location: Location,
}

impl Disparity {
    pub fn new(message: &str, location: Location) -> Self {
        Disparity {
            message: message.to_string(),
            location,
        }
    }
    pub fn to_list(self) -> DisparityList {
        let mut list = DisparityList::new();
        list.push(self);
        list
    }
}

impl Display for Disparity {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.location.is_empty() {
            writeln!(f, "{}", self.message)
        } else {
             write!(f, "{}\n↳ {}", self.location, self.message)
        }
    }
}



#[derive(Debug)]
pub struct DisparityList {
    pub inner: Vec<Disparity>,
}

impl DisparityList {
    pub fn new() -> Self {
        DisparityList { inner: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn push(&mut self, dis: Disparity) {
        self.inner.push(dis)
    }

    pub fn option_push(&mut self, dis: Option<Disparity>) -> bool {
        dis.map(|disparity| self.push(disparity)).is_some()
    }

    pub fn merge(&mut self, other_list: DisparityList) {
        other_list
            .inner
            .into_iter()
            .for_each(|disparity| self.push(disparity));
    }
}

impl Display for DisparityList {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let text_list: Vec<String> = self.inner.iter().map(|dis| format!("{}\n", dis)).collect();
        let text = text_list.join("\n");
        f.write_str(&text)
    }
}

#[derive(Debug)]
pub enum Extremum {
    Minimum(f32),
    Maximum(f32),
}

#[derive(Clone, Copy)]
pub struct ExtremumFinder {
    t_m_2: Option<f32>,
    t_m_1: Option<f32>,
    t: Option<f32>,
}

impl ExtremumFinder {
    pub fn new() -> Self {
        ExtremumFinder {
            t_m_2: None,
            t_m_1: None,
            t: None,
        }
    }
    pub fn push(&mut self, v: f32) -> Option<Extremum> {
        self.t_m_2 = self.t_m_1;
        self.t_m_1 = self.t;
        self.t = Some(v);

        if let (Some(a), Some(b), Some(c)) = (self.t_m_2, self.t_m_1, self.t) {
            if a < b && b > c {
                Some(Extremum::Maximum(b))
            } else if a > b && b < c {
                Some(Extremum::Minimum(b))
            } else {
                None
            }
        } else {
            None
        }
    }
}

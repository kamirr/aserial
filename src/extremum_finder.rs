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
    t_p_1: Option<f32>,
    t_p_2: Option<f32>,
}

impl ExtremumFinder {
    pub fn new() -> Self {
        ExtremumFinder {
            t_m_2: None,
            t_m_1: None,
            t: None,
            t_p_1: None,
            t_p_2: None,
        }
    }
    pub fn push(&mut self, v: f32) -> Option<Extremum> {
        self.t_m_2 = self.t_m_1;
        self.t_m_1 = self.t;
        self.t = self.t_p_1;
        self.t_p_1 = self.t_p_2;
        self.t_p_2 = Some(v);

        if let (Some(m2), Some(m1), Some(t0), Some(p1), Some(p2))
            = (self.t_m_2, self.t_m_1, self.t, self.t_p_1, self.t_p_2) {
            if m2 < m1 && m1 < t0 && t0 > p1 && t0 > p2 {
                Some(Extremum::Maximum(t0))
            } else if m2 > m1 && m1 > t0 && t0 < p1 && t0 < p2 {
                Some(Extremum::Minimum(t0))
            } else {
                None
            }
        } else {
            None
        }
    }
}

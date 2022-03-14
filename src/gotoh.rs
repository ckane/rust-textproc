use min_max::*;
use std::fmt;

#[derive(Debug)]
pub struct  GotohInstance {
    gap_creation: isize,
    gap_extend: isize,
    subst: isize,
    width: usize,
    height: usize,
    d_mat: Box<[isize]>,
    p_mat: Box<[isize]>,
    q_mat: Box<[isize]>,
}

impl fmt::Display for GotohInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for j in 0..self.height+1 {
            for i in 0..self.width+1 {
                let val = self.d_mat[i+(self.width+1)*j];
                if val == isize::MIN {
                    write!(f, " inf");
                } else {
                    write!(f, "{:4}", val);
                }
            }
            write!(f, "\n");
        }
        write!(f, "\n");

        for j in 0..self.height+1 {
            for i in 0..self.width+1 {
                let val = self.p_mat[i+(self.width+1)*j];
                if val == isize::MIN {
                    write!(f, " inf");
                } else {
                    write!(f, "{:4}", val);
                }
            }
            write!(f, "\n");
        }
        write!(f, "\n");

        for j in 0..self.height+1 {
            for i in 0..self.width+1 {
                let val = self.q_mat[i+(self.width+1)*j];
                if val == isize::MIN {
                    write!(f, " inf");
                } else {
                    write!(f, "{:4}", val);
                }
            }
            write!(f, "\n");
        }
        write!(f, "")
    }
}

impl GotohInstance {
    pub fn new(gap_creation_penalty: isize, gap_extension_penalty: isize, subst_penalty: isize) -> Self {
        let me: Self = Self {
            height: 0,
            width: 0,
            d_mat: Box::new([]),
            p_mat: Box::new([]),
            q_mat: Box::new([]),
            gap_creation: gap_creation_penalty,
            gap_extend: gap_extension_penalty,
            subst: subst_penalty,
        };
        me
    }

    pub fn init(self: &mut Self, s1: &String, s2: &String) -> isize {
        self.width = s1.len();
        self.height = s2.len();

        let min = isize::MIN;

        self.d_mat = vec![0; (self.width + 1) * (self.height + 1)].into_boxed_slice();
        self.p_mat = vec![0; (self.width + 1) * (self.height + 1)].into_boxed_slice();
        self.q_mat = vec![0; (self.width + 1) * (self.height + 1)].into_boxed_slice();

        self.d_mat[0] = 0;
        self.p_mat[0] = min;
        self.q_mat[0] = min;

        for i in 1..self.width+1 {
            self.d_mat[i] = min;
            self.p_mat[i] = -self.gap_creation - self.gap_extend * ((i - 1) as isize);
            self.q_mat[i] = min;
            self.q_mat[i + self.width + 1] = -self.gap_creation;
        };

        for j in 1..self.height+1 {
            self.d_mat[j*(self.width + 1)] = min;
            self.p_mat[j*(self.width + 1)] = min;
            self.p_mat[1 + j*(self.width + 1)] = -self.gap_creation;
            self.q_mat[j*(self.width + 1)] = -self.gap_creation - self.gap_extend * ((j - 1) as isize);
        };

        let s1b = s1.as_bytes();
        let s2b = s2.as_bytes();

        //println!("{}", self);

        for i in 1..self.width+1 {
            for j in 1..self.height+1 {
                let sim_val: isize = if s1b[i-1] == s2b[j-1] {
                    self.subst
                } else {
                    0
                };
                //println!("Compare {} to {} = {}", s1b[i-1], s2b[j-1], sim_val);

                self.d_mat[i + (self.width + 1)*j] = max!(
                    if self.d_mat[i - 1 + (self.width + 1)*(j - 1)] != min {
                        self.d_mat[i - 1 + (self.width + 1)*(j - 1)] + sim_val
                    } else {
                        min
                    },
                    if self.p_mat[i - 1 + (self.width + 1)*(j - 1)] != min {
                        self.p_mat[i - 1 + (self.width + 1)*(j - 1)] + sim_val
                    } else {
                        min
                    },
                    if self.q_mat[i - 1 + (self.width + 1)*(j - 1)] != min {
                        self.q_mat[i - 1 + (self.width + 1)*(j - 1)] + sim_val
                    } else {
                        min
                    }
                );

                self.p_mat[i + (self.width + 1)*j] = max!(
                    if self.d_mat[i - 1 + (self.width + 1)*j] != min {
                        self.d_mat[i - 1 + (self.width + 1)*j] - self.gap_creation
                    } else {
                        min
                    },
                    if self.p_mat[i - 1 + (self.width + 1)*j] != min {
                        self.p_mat[i - 1 + (self.width + 1)*j] - self.gap_extend
                    } else {
                        min
                    }
                );

                self.q_mat[i + (self.width + 1)*j] = max!(
                    if self.d_mat[i + (self.width + 1)*(j-1)] != min {
                        self.d_mat[i + (self.width + 1)*(j-1)] - self.gap_creation
                    } else {
                        min
                    },
                    if self.q_mat[i + (self.width + 1)*(j-1)] != min {
                        self.q_mat[i + (self.width + 1)*(j-1)] - self.gap_extend
                    } else {
                        min
                    }
                );
            };
        };

        //println!("{:?}", self.d_mat);
        //println!("{:?}", self.p_mat);
        //println!("{:?}", self.q_mat);
        //println!("{}", self);

        max!(self.d_mat[self.width + (self.width + 1)*(self.height)],
             self.p_mat[self.width + (self.width + 1)*(self.height)],
             self.q_mat[self.width + (self.width + 1)*(self.height)])
    }
}

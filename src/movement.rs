#[derive(Copy, Clone, PartialEq)]
pub struct Move{
	pub from: Position,
	pub to: Position,
	value: Option<Score>
}

pub type Score = u32;

#[derive(Copy, Clone, PartialEq)]
pub struct Position{
	pub x: usize,
	pub y: usize
}

impl Move{
	//Lager et nytt move. NB! Denne bryr seg kun om koordinater, der Origo er oppe til venstre.
	//Dermed er e1->e2 det samme som (4, 7, 4, 6).
	pub fn new(filefrom: usize, rankfrom: usize, fileto: usize, rankto: usize) -> Self{
		Move{from: Position{x: filefrom, y: rankfrom}, to: Position{x: fileto, y: rankto}, value: None}
	}

	pub fn value(&self) -> Score{
		match self.value{
			None    => { panic!("This move has no associated value."); }
			Some(v) => v
		}
	}
}
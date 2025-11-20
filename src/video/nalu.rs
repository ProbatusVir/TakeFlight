pub enum NriPriority
{
	Lowest,
	Low,
	High,
	Highest,
}

pub trait Nal
{

	fn is_forbidden(&self) -> bool;
	fn nri(&self) -> NriPriority;
	/// This will change to an enum once it's better understood.
	fn unit_type(&self) -> u8;
}

impl Nal for u8
{
	fn is_forbidden(&self) -> bool {
		const FORBIDDEN_BIT : u8 = 0b1000_0000;

		let unit_type = self.unit_type();
		let nri_must_be_zero = unit_type == 6 || (unit_type >= 9 && unit_type <= 12);

		(self & FORBIDDEN_BIT == 0) || (nri_must_be_zero && *self == self.nri() as Self)
	}

	fn nri(&self) -> NriPriority {
		const NRI_BITMASK : u8 = 0b0110_0000;
		let selection = self & NRI_BITMASK;
		match selection {
			0b0110_0000	=> { NriPriority::Highest	}
			0b0100_0000	=> { NriPriority::High		}
			0b0010_0000	=> { NriPriority::Low		}
			_			=> { NriPriority::Lowest	} // 0 is the only other possible variant, but the computer doesn't know that...
		}
	}

	fn unit_type(&self) -> u8 {
		self & 0b0001_1111
	}
}
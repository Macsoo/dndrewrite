#[derive(Debug, Copy, Clone)]
pub enum Size {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

pub struct Hp {
    current: u16,
    max: u16,
    temporary: u16,
}

impl Hp {
    pub fn damage(&mut self, amount: u16) -> bool {
        let new_amount = amount.saturating_sub(self.temporary);
        self.temporary = self.temporary - (amount - new_amount);
        if let Some(new) = self.current.checked_sub(new_amount) {
            self.current = new;
            new == 0
        } else {
            self.current = 0;
            true
        }
    }

    pub fn heal(&mut self, amount: u16) {
        self.current = self.max.min(self.current + amount);
    }

    pub fn set_temporary(&mut self, amount: u16) {
        self.temporary = amount;
    }
}
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub enum Animal {
    Dog,
    Cat,
}

// Though it has the proc-macro, we drop the variant
// literals if there is not a repr type defined
#[derive(uniffi::Enum)]
pub enum AnimalNoReprInt {
    Dog = 3,
    Cat = 4,
}

#[repr(u8)]
#[derive(uniffi::Enum)]
pub enum AnimalUInt {
    Dog = 3,
    Cat = 4,
}

#[repr(u64)]
#[derive(uniffi::Enum)]
pub enum AnimalLargeUInt {
    Dog = 4294967298, // u32::MAX as u64 + 3
    Cat = 4294967299, // u32::MAX as u64 + 4
}

#[repr(i8)]
#[derive(Debug, uniffi::Enum)]
pub enum AnimalSignedInt {
    Dog = -3,
    Cat = -2,
    Koala,   // -1
    Wallaby, // 0
    Wombat,  // 1
}

#[uniffi::export]
fn get_animal(a: Option<Animal>) -> Animal {
    a.unwrap_or(Animal::Dog)
}

uniffi::include_scaffolding!("enum_types");

#[cfg(test)]
mod test {
    use crate::AnimalSignedInt;

    #[test]
    fn check_signed() {
        assert_eq!(AnimalSignedInt::Koala as i8, -1);
        assert_eq!(AnimalSignedInt::Wallaby as i8, 0);
        assert_eq!(AnimalSignedInt::Wombat as i8, 1);
    }
}

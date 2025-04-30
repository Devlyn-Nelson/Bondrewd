use bondrewd::Bitfields;

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(id_bit_length = 3, default_endianness = "be")]
enum EyeColor {
    Blue,
    Green,
    Brown,
    Other {
        #[bondrewd(capture_id)]
        test: u8,
    },
}

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(default_endianness = "be")]
struct PersonParts {
    head: bool,
    #[bondrewd(bit_length = 2)]
    shoulders: u8,
    #[bondrewd(bit_length = 2)]
    knees: u8,
    #[bondrewd(bit_length = 4)]
    toes: u8,
}

#[derive(Bitfields, Clone, Eq, PartialEq, Debug)]
#[bondrewd(default_endianness = "be")]
struct Person {
    // Name is english only?
    #[bondrewd(element_byte_length = 2)]
    name: [char; 32],
    // Age of Person Nobody lives past 127
    #[bondrewd(bit_length = 7)]
    age: u8,
    // Eyes have color
    #[bondrewd(bit_length = 3)]
    eye_color: EyeColor,
    // Standard components
    #[bondrewd(bit_length = 9)]
    parts: PersonParts,
    // How many times they have blinked each of their eyes
    #[bondrewd(bit_length = 60)]
    blinks: u64,
}

#[derive(Debug, thiserror::Error)]
#[cfg(all(feature = "std", feature = "dyn_fns"))]
enum MyError {
    #[error(transparent)]
    LengthError(#[from] bondrewd::BitfieldLengthError),
}

fn main() -> anyhow::Result<()> {
    let person = Person {
        name: ['a'; 32],
        age: 27,
        eye_color: EyeColor::Blue,
        parts: PersonParts {
            head: true,
            shoulders: 2,
            knees: 2,
            toes: 10,
        },
        blinks: 10_000_000_000,
    };

    // Get bitfield form of `person`.
    let bytes = person.clone().into_bytes();

    // Test reconstructing the output works and verify it is correct.
    assert_eq!(Person::from_bytes(bytes), person);

    // Test changing the output works as expected. This will be our target.
    let target_changes = Person {
        name: ['b'; 32],
        age: 72,
        eye_color: EyeColor::Green,
        parts: PersonParts {
            head: false,
            shoulders: 1,
            knees: 3,
            toes: 7,
        },
        blinks: 5,
    };

    // Get `person` as bytes again, which has different values than out target.
    let mut bytes = person.into_bytes();

    #[cfg(not(feature = "dyn_fns"))]
    {
        // Change output.
        Person::write_name(&mut bytes, target_changes.name);
        Person::write_age(&mut bytes, target_changes.age);
        Person::write_eye_color(&mut bytes, target_changes.eye_color.clone());
        Person::write_parts(&mut bytes, target_changes.parts.clone());
        Person::write_blinks(&mut bytes, target_changes.blinks);

        // Verify.
        assert_eq!(Person::read_name(&bytes), target_changes.name);
        assert_eq!(Person::read_age(&bytes), target_changes.age);
        assert_eq!(
            Person::read_eye_color(&bytes),
            target_changes.eye_color.clone()
        );
        assert_eq!(Person::read_parts(&bytes), target_changes.parts.clone());
        assert_eq!(Person::read_blinks(&bytes), target_changes.blinks);
    }

    #[cfg(feature = "dyn_fns")]
    {
        // Change some of the output via individual write_slice function.
        Person::write_slice_name(&mut bytes[..64], target_changes.name)?;
        Person::write_slice_age(&mut bytes[..71], target_changes.age)?;

        // Verify. (notice i only provided the required amount of bytes to read the field i want)
        assert_eq!(Person::read_slice_name(&bytes[..64])?, target_changes.name);
        assert_eq!(Person::read_slice_age(&bytes[..71])?, target_changes.age);

        // Change the rest of the output with the Checked structure.
        let mut checked = Person::check_slice_mut(&mut bytes)?;
        checked.write_eye_color(target_changes.eye_color.clone());
        checked.write_parts(target_changes.parts.clone());
        checked.write_blinks(target_changes.blinks);

        // Verify more.
        assert_eq!(
            Person::read_eye_color(&bytes),
            target_changes.eye_color.clone()
        );
        assert_eq!(Person::read_parts(&bytes), target_changes.parts.clone());
        assert_eq!(Person::read_blinks(&bytes), target_changes.blinks);
    }

    // Verify Harder.
    let new_person = Person::from_bytes(bytes);
    assert_eq!(new_person, target_changes);
    Ok(())
}

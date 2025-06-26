use bondrewd::Bitfields;

#[derive(Bitfields, Debug, Clone)]
#[bondrewd(id_bit_length = 8)]
enum TupleEnum {
    One(u8),
    Two(u8),
    Invalid(#[bondrewd(capture_id)] u8, u8)
}

fn main() {
    let one: TupleEnum = TupleEnum::One(1);
    let two = TupleEnum::Two(2);
    let err = TupleEnum::Invalid(4, 3);

    let mut one_bytes = one.clone().into_bytes();
    let mut two_bytes = two.clone().into_bytes();
    let mut err_bytes = err.clone().into_bytes();

    assert_eq!(one_bytes, [0,1]);
    assert_eq!(two_bytes, [1,2]);
    assert_eq!(err_bytes, [4,3]);

    // i am rotating the values so that `one` gets `two's` value, `two` gets `err's`, and `err` gets `one's`.
    TupleEnum::write_one_field_1(&mut one_bytes, 2);
    TupleEnum::write_two_field_1(&mut two_bytes, 3);
    TupleEnum::write_invalid_field_1(&mut err_bytes, 1);

    // rotating the id's the same way the values were rotated.
    TupleEnum::write_variant_id(&mut one_bytes, 1);
    TupleEnum::write_variant_id(&mut two_bytes, 4);
    TupleEnum::write_variant_id(&mut err_bytes, 0);

    // because we rotated the bytes above using the write function we should name to reconstructed
    // structures as they should be based oin the actual values.
    // 
    // ex.
    // `two_bytes` was set to the same values as `one` so `two_bytes` will become `new_one` and be checked
    // against `one`.
    let _new_one: TupleEnum = TupleEnum::from_bytes(two_bytes);
    let _new_two = TupleEnum::from_bytes(err_bytes);
    let _new_err = TupleEnum::from_bytes(one_bytes);


    assert!(matches!(one, _new_one));
    assert!(matches!(two, _new_two));
    assert!(matches!(err, _new_err));
}


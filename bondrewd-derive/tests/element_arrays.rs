use bondrewd::Bitfields;

#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be")]
struct SimpleWithArray {
    #[bondrewd(bit_length = 4)]
    one: u8,
    #[bondrewd(element_bit_length = 1)]
    two: [bool; 5],
    #[bondrewd(bit_length = 7)]
    three: u8,
}
#[test]
fn to_bytes_simple_with_element_array_spanning() -> anyhow::Result<()> {
    let simple = SimpleWithArray {
        one: 0,
        two: [true, false, true, false, true],
        three: 0,
    };
    assert_eq!(SimpleWithArray::BYTE_SIZE, 2);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 2);
    assert_eq!(bytes[0], 0b0000_1010);
    assert_eq!(bytes[1], 0b1000_0000);
    #[cfg(feature = "dyn_fns")]
    {
        //peeks
        assert_eq!(simple.one, SimpleWithArray::read_slice_one(&bytes)?);
        assert_eq!(simple.two, SimpleWithArray::read_slice_two(&bytes)?);
        assert_eq!(simple.three, SimpleWithArray::read_slice_three(&bytes)?);
    }

    // from_bytes
    let new_simple = SimpleWithArray::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}
#[derive(Bitfields, Clone, PartialEq, Eq, Debug)]
#[bondrewd(default_endianness = "be", reverse, read_from = "lsb0", dump)]
struct SimpleWithArrayOrderTest {
    test: [u8; 6],
}
#[test]
fn array_order_test() -> anyhow::Result<()> {
    let simple = SimpleWithArrayOrderTest {
        test: [0, 42, 0, 200, 0, 56],
    };
    assert_eq!(SimpleWithArrayOrderTest::BYTE_SIZE, 6);
    let bytes = simple.clone().into_bytes();
    assert_eq!(bytes.len(), 6);
    assert_eq!(bytes[0], simple.test[0]);
    assert_eq!(bytes[1], simple.test[1]);
    assert_eq!(bytes[2], simple.test[2]);
    assert_eq!(bytes[3], simple.test[3]);
    assert_eq!(bytes[4], simple.test[4]);
    assert_eq!(bytes[5], simple.test[5]);
    #[cfg(feature = "dyn_fns")]
    {
        //peeks
        assert_eq!(
            simple.one,
            SimpleWithArrayOrderTest::read_slice_one(&bytes)?
        );
        assert_eq!(
            simple.two,
            SimpleWithArrayOrderTest::read_slice_two(&bytes)?
        );
        assert_eq!(
            simple.three,
            SimpleWithArrayOrderTest::read_slice_three(&bytes)?
        );
    }

    // from_bytes
    let new_simple = SimpleWithArrayOrderTest::from_bytes(bytes);
    assert_eq!(simple, new_simple);
    Ok(())
}

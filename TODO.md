


# Must Fix

- fix `ne` multi-byte.
- verify overlap protection works.
  - [ ] enums
  - [ ] struct
- verify sign_fix works for signed number.
- verify a multi-byte rust type can be given less than 9 bits. currently only u8 and i8 fields will work here. verify bool works it might. amount of zeros to have for the left mask. (left mask meaning a mask to keep data on the left)
- test to see that the struct enforcement error are nice and accurate.

# Features

- add option to put the id field at the tail instead of the head of the bits.
- add `FillTo` which should make a fill field that fills the remaining unused bits to specified amount. if the current used bits amount goes over the fill-to amount specified throw error.
- add `into_object` function to checked structures, this would be an alternative to using `Bitfields::from_bytes` but from a checked slice.
- Make a way for the Id of a enum to be received via another field when using nested enums.
- Allow id_field in enums to be other primitive types that can be matched against.
- Zero shifts happen still. Please insure these don't get output.

# Optimizations

- [ ] Make even bytes optimizations. when bit fields are not necessary we could optimize things by using copy from slice.

# Overhauls

- [x] Make bondrewd builder
  - [x] Create `Solved` layer that solves the builder into basic pre-bitfield-derive information.
    - [ ] this could be used to calculate final masks then create the derive code.
    - [ ] this could also be used to generate the same mask data and use it at runtime.
  - [ ] Create `SolvedMasks` layer that is just the calculated data needed to create the derive functions but could also be used at runtime or make derive functions. the functions in the solved layer would simple utilize this layer as convenance and to reduce memory usage while not using the built model.
  - [ ] Separate the fields start and end indices and have them be an enum type with allows for `dynamic` or `static` values. This is to create Dynamic systems.
    - [ ] When the starting and ending bit indices are `static` we can use the standard bondrewd systems we that have always existed which create infallible bit extraction and can have a `Bitfields` implementation.
    - [ ] We need to create a runtime version of the derive function that don't use pre-calculated values, but rather feed in at least the starting index then either calculate the end or also pass it in. then do runtime bitfield extraction. this is for `dynamic` fields
      - NOTE: Any `static` fields before a `dynamic` should still use compile time calculated extraction.
      - [ ] When a starting index is `dynamic` we use the previous fields ending index to determine the starting index, which might sound like how it is already done, BUT the read and write functions would require the starting index to be provided during runtime to determine where it starts.
        - NOTE: should be apparent that the first field should never have a `dynamic` starting index.
      - [ ] When ending index is `static` we calculate the value at runtime.
      - [ ] When ending index is `dynamic` it is required that a previous field be provided to get a length from. which will make the read and write functions also require a ending index to provided during runtime to determine where it ends.
    - NOTE: When ANY field has a `dynamic` starting or ending bit index, we lose the ability to implement `Bitfields` but `BitfieldsDyn` would still be an option.
  
# Think On

- I might want `FillBits::Auto` to replace `FillBits::None`.

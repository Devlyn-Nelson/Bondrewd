- BIT_SIZE doesn't include fill.

- fill_bytes no longer fill up until that many bytes but appends bytes.

- in the old enum_derives example `CenteredInvalid` didn't need to define the variants id value because it was the 3rd variant meaning it would automatically be assigned 2. now due to the invalid variant getting removed from the variant list and handled separately the id needs to get specified because Invalid will always be processed last.
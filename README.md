# Valley

A sierra decompiler into a "verbose" version of Cairo.

The decompiled code is not valid cairo but it may help in understanding the original code.

> [!IMPORTANT]
> Currently in very very very early development, in my free time.

# Example
```cairo
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}


pub fn sub(a: u32, b: u32, c: u32) -> u32 {
    a - b - c
}
```

```
valley program.sierra
```

Generates:

```cairo

pub fn func_10915670729824419620(v0: RangeCheck, v1: u32, v2: u32) -> (RangeCheck, Enum<(u32), ((), Array<felt252>)>) {
    let (v6 : u32, v6_overflowed: bool) = v1 + v2;
    if !v6_overflowed {
        let v3: RangeCheck = v0;
        let v7: (u32) = Struct {
        	field_0: v4,
        };
        let v8: Enum<(u32), ((), Array<felt252>)> = Enum::Variant0(v7);

        return v3, v8;
    } else {
        let v5: RangeCheck = v0;
        drop(v6);
        let mut v9: Array<felt252> = Array::new();
        let v10: felt252 = 155785504323917466144735657540098748279;
        v9.append(v10);
        let mut v11: Array<felt252> = v9;
        let v12: () = Struct {
        };
        let v13: ((), Array<felt252>) = Struct {
        	field_0: v12,
        	field_1: v11,
        };
        let v14: Enum<(u32), ((), Array<felt252>)> = Enum::Variant1(v13);

        return v5, v14;
    }
}

pub fn func_2293170801303492997(v0: RangeCheck, v1: u32, v2: u32, v3: u32) -> (RangeCheck, Enum<(u32), ((), Array<felt252>)>) {
    let (v7 : u32, v7_overflowed: bool) = v1 - v2;
    if !v7_overflowed {
        let v4: RangeCheck = v0;
        let (v11 : u32, v11_overflowed: bool) = v5 - v3;
        if !v11_overflowed {
            let v8: RangeCheck = v4;
            let v12: (u32) = Struct {
            	field_0: v9,
            };
            let v13: Enum<(u32), ((), Array<felt252>)> = Enum::Variant0(v12);

            return v8, v13;
        } else {
            let v10: RangeCheck = v4;
            drop(v11);
            let mut v14: Array<felt252> = Array::new();
            let v15: felt252 = 155785504329508738615720351733824384887;
            v14.append(v15);
            let mut v16: Array<felt252> = v14;
            let v17: () = Struct {
            };
            let v18: ((), Array<felt252>) = Struct {
            	field_0: v17,
            	field_1: v16,
            };
            let v19: Enum<(u32), ((), Array<felt252>)> = Enum::Variant1(v18);

            return v10, v19;
        }
    } else {
        let v6: RangeCheck = v0;
        drop(v7);
        drop(v3);
        let mut v20: Array<felt252> = Array::new();
        let v21: felt252 = 155785504329508738615720351733824384887;
        v20.append(v21);
        let mut v22: Array<felt252> = v20;
        let v23: () = Struct {
        };
        let v24: ((), Array<felt252>) = Struct {
        	field_0: v23,
        	field_1: v22,
        };
        let v25: Enum<(u32), ((), Array<felt252>)> = Enum::Variant1(v24);

        return v6, v25;
    }
}


```

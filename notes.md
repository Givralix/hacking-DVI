- ~if data enable we send pixels otherwise we send control data.
- each color channel is on its own cable

## Algorithm

reference: [DVI specsheet](https://glenwing.github.io/docs/DVI-1.0.pdf)

`D` - input pixel data
`C1`, `C0` - control data for the channel
`DE` - data enable
`N_0{x}` - number of 0s in x (we use `count_zeros(x)`)
`N_1{x}` - number of 1s in x (we use `count_ones(x)`)
`cnt{t}` - positive value if more 1s have been transmitted and negative if more 0s have been transmitted
`q_out` - 10 bits of encoded output value (one channel)

---

### DE is HIGH

```
q_m[0] = D[0]
if count_ones(D) > 4 || (count_ones(D) == 4 && D[0] == 0)
    for i in 1..=7
        q_m[i] = q_m[i-1] XNOR D[i]
    q_m[8] = 0
else
    for i in 1..=7
        q_m[i] = q_m[i-1] XOR D[i]
    q_m[8] = 1

if cnt{t-1} == 0 || count_ones(q_m[0:7]) == count_zeros(q_m[0:7])
    q_out[9] = !q_m[8]
    q_out[8] = q_m[8]
    if q_m[8] == 1
        q_out[0:7] = q_m[0:7]
    else
        q_out[0:7] = !q_m[0:7]
    
    if q_m[8] == 0
        cnt{t} = cnt{t-1} + count_zeros(q_m[0:7]) - count_ones[0:7]
    else
        cnt{t} = cnt{t-1} - count_zeros(q_m[0:7]) + count_ones[0:7]

else 
    if (cnt(t-1) > 0 && (count_ones(q_m[0:7]) > count_zeros(q_m[0:7])))
    || (cnt(t-1) < 0 && (count_ones(q_m[0:7]) < count_zeros(q_m[0:7])))
        // there were more ones (or zeros) and there still are
        q_out[9] = 1
        q_out[8] = q_m[8]
        q_out[0:7] = !q_m[0:7]
        cnt{t} = cnt{t-1} + 2*(q_m[8]) - count_ones(q_m[0:7]) + count_zeros(q_m[0:7])
    else
        q_out[9] = 0
        q_out[8] = q_m[8]
        q_out[0:7] = q_m[0:7]
        cnt{t} = cnt{t-1} - 2*(!q_m[8]) + count_ones(q_m[0:7]) - count_zeros(q_m[0:7])
```

### DE is LOW

```
Cnt(t) = 0;
case (C1, C0)
  00:   q_out[0:9] = 0010101011;
  01:   q_out[0:9] = 1101010100;
  10:   q_out[0:9] = 0010101010;
  11:   q_out[0:9] = 1101010101;
endcase
```
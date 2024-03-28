( Test of [DEFINED] )
s" Preamble.  SHOULD get compiled."

[DEFINED] t01 [IF]
    VARIABLE t01_var
    s" This SHOULD get compiled"
    ( Another test happens )
    [DEFINED] t01_not_defined [IF]
        s" This should NOT get compiled"
    [ELSE]
        s" t01_not_defined else SHOULD get compiled"
    [THEN]
    s" t01_not_defined then SHOULD get compiled"
[ELSE]
    s" t01 else should NOT get compiled"
    [DEFINED] t01 [IF]
        s" t01 else+if nested should NOT get compiled"
    [ELSE]
        s" t01 else+else nested should NOT get compiled"
    [THEN]
    s" t01 else post should NOT get compiled"
[THEN]
s" Postamble.  SHOULD get compiled."

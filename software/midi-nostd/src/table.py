import math  

def do_square( wave_range ):
    return -1.0 if wave_range < .5 else 1.0

def do_sine( wave_range ):
    return math.sin(wave_range * math.pi * 2.0) 

def do_sawtooth( wave_range ):
    rval = wave_range * 2;
    if rval > 1:
        rval = rval - 2
    return rval

def do_triangle( wave_range ):
    rval = 0;
    if wave_range < .25:
        rval = wave_range * 4.0;
    elif wave_range < .75:
        rval = 1.0 - (wave_range-.25) * 4.0;
    else:
        rval = (wave_range - 1) * 4.0;

    return rval

def do_table_entry( wave_function, idx ):
    wave_range = idx / 1024.0
    wave_domain = wave_function( wave_range )
    return str(int( wave_domain * 32768))

def do_table_row( wave_function, big_idx ):
    row_numbers = [do_table_entry( wave_function, big_idx + i) for i in range( 8 ) ]
    return "    " + ",".join( row_numbers )

def output_table( function_name, wave_function ): 
    row_lines = [do_table_row( wave_function, i) for i in range( 0, 1024, 8 ) ]
    print( "pub const", function_name, ": [i32; WAVE_TABLE_SIZE] = [" )
    print( ",\n".join( row_lines ))
    print( "];" )
    print( "" )

print("pub const WAVE_TABLE_SIZE: usize=1024;\n");
print("pub const WAVE_TABLE_SIZE_U32: u32 =1024;\n");

output_table( "TRIANGLE_WAVE",  do_triangle     )
output_table( "SINE_WAVE",      do_sine         )
output_table( "SQUARE_WAVE",    do_square       )
output_table( "SAWTOOTH_WAVE",  do_sawtooth     )


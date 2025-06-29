import math  

def do_square( wave_range ):
    return 0.0 if wave_range < .5 else 1.0

def do_sine( wave_range ):
    return math.sin(wave_range * math.pi * 2.0) * .5 + .5

def do_sawtooth( wave_range ):
    return wave_range

def do_triangle( wave_range ):
    return wave_range * 2.0 if wave_range < .5 else (1.0-wave_range) * 2.0

def do_table_entry( wave_function, idx ):
    wave_range = idx / 1024.0
    wave_domain = wave_function( wave_range )
    return str(int( wave_domain * 256))

def do_table_row( wave_function, big_idx ):
    row_numbers = [do_table_entry( wave_function, big_idx + i) for i in range( 16 ) ]
    return "    " + ",".join( row_numbers )

def output_table( function_name, wave_function ): 
    row_lines = [do_table_row( wave_function, i) for i in range( 0, 1024, 16 ) ]
    print( "#[allow(unused)]" )
    print( "const", function_name, ": [u32; WAVE_TABLE_SIZE] = [" )
    print( ",\n".join( row_lines ))
    print( "];" )
    print( "" )

print( "#[allow(unused)]" )
print("pub const WAVE_TABLE_SIZE: usize=1024;\n");

output_table( "TRIANGLE_WAVE",  do_triangle     )
output_table( "SINE_WAVE",      do_sine         )
output_table( "SQUARE_WAVE",    do_square       )
output_table( "SAWTOOTH_WAVE",  do_sawtooth     )


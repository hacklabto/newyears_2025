import math

max_range = int(.201 * 1024)

print("const TAN_TABLE: [i64;" + str(max_range) +"] = [")
for i in range( 0, max_range ):
    print("    "+str(int(math.tan( math.pi * float(i) /1024.0) * float(1<<31)) )+",")
print("];")


runtimes = {}

def parse_line(line):
    try:
        id_func, runtime = line.split(':')
        
        _id, func = id_func.split(',')

        if not func in runtimes:
            runtimes[func] = {}
        if not _id in runtimes[func]:
            runtimes[func][_id] = []

        runtimes[func][_id].append(runtime)
    except Exception:
        pass


file = open(input("Enter file path: "))
for line_num, line in enumerate(file):
    if line_num == 0:
        continue
    if line_num == 1:
        continue
    parse_line(line)

for func in runtimes:
    for _id in runtimes[func]:
        total = 0
        avg = 0
        for runtime in runtimes[func][_id]:
            total += 1
            avg += int(runtime)
        print("{" + _id + "} " + func + ": " + str(avg/total) + "ns | " + str(total) + " times")

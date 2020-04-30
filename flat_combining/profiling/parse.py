import sys
from os import walk

files = []
for (dirpath, dirnames, filenames) in walk(sys.argv[1]):
    files.extend(filenames)
    break

print("File: " + str(files))

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


for _file in files:
    if sys.argv[2] in _file:
        print("---", _file, "---")
        for line_num, line in enumerate(open(sys.argv[1] + '/' + _file)):
            if line_num == 0:
                continue
            if line_num == 1:
                continue
            parse_line(line)

        for func in runtimes:
            all_total = 0
            all_thread_avg = 0
            max_thread_time = 0
            for _id in runtimes[func]:
                total = 0
                avg = 0
                for runtime in runtimes[func][_id]:
                    total += 1
                    avg += int(runtime)
                    max_thread_time = max(max_thread_time, int(runtime))

                all_total += total
                all_thread_avg += avg

                # print("{" + _id + "} " + func + ": " + str(avg/total) + "ns | " + str(total) + " times")
            print(func + " avg: " + str(all_thread_avg/all_total) + "ns | " + str(all_total) + " times")
            print(func + " max: " + str(max_thread_time) + "ns | " + str(all_total) + " times")
        runtimes = {}

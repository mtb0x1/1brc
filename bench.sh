#!/bin/bash

#/!\ assumption measurement file name is measurement_data_SIZE.txt, or defaults to measurement_data.txt

if [ "$#" -eq 0 ]; then
    # If not provided, set default value
    output_file="benchmark.json"
    data_file="measurement_data.txt"
else
    # If provided, use the provided value
    output_file="benchmark_$1.json"
    data_file="measurement_data_$1.txt"
fi

hyperfine   --export-json="$output_file" \
            --warmup=3 \
            --time-unit=millisecond \
            --command-name="[V1]target/release/1brc ${data_file}" "target/release/1brc ${data_file}" \
            --command-name="[V0]target/release/Lucretiel_1brc ${data_file}" "target/release/Lucretiel_1brc ${data_file}"

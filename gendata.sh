#!/bin/bash

output_file="measurement_data.txt"
total_lines=1000000000
num_processes=10  # Adjust this based on your system's capabilities

generate_measurement() {
    printf "%.1f" $(awk -v min=0 -v max=40 'BEGIN{srand(); print min+rand()*(max-min)}')
}

generate_station_name() {
    cities=("Hamburg" "Bulawayo" "Palembang" "St. John's" "Cracow" "Bridgetown" "Istanbul" "Roseau" "Conakry")
    index=$(($RANDOM % ${#cities[@]}))
    echo "${cities[$index]}"
}

generate_lines() {
    local start=$1
    local end=$2

    for ((i=start; i<=end; i++)); do
        station_name=$(generate_station_name)
        measurement=$(generate_measurement)
        echo "${station_name};${measurement}"
    done
}

# Calculate lines per process
lines_per_process=$((total_lines / num_processes))

# Run processes in parallel
for ((p=0; p<num_processes-1; p++)); do
    start_line=$((p * lines_per_process + 1))
    end_line=$((start_line + lines_per_process - 1))

    generate_lines "$start_line" "$end_line" >> "$output_file" &
done

# Run the last process separately to handle any remaining lines
start_line=$(( (num_processes-1) * lines_per_process + 1 ))
generate_lines "$start_line" "$total_lines" >> "$output_file" &

# Wait for all background processes to finish
wait

echo "File generated: $output_file"
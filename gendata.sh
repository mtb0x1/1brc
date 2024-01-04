#!/bin/bash

output_file="measurement_data.txt"
# Check if the total_lines argument is provided
if [ "$#" -eq 0 ]; then
  # If not provided, set default value
  total_lines=1000000000
else
  # If provided, use the provided value
  total_lines="$1"
  output_file="measurement_data_$1.txt"
fi

num_processes=2  # Adjust this based on your system's capabilities

generate_measurement() {
    printf "%.1f" $(awk -v min=0 -v max=40 'BEGIN{srand(); print min+rand()*(max-min)}')
}

generate_station_name() {
    cities=("Abha" "Abidjan" "Abéché" "Accra" "Addis Ababa" "Adelaide" "Aden" "Ahvaz" "Albuquerque" "Alexandra"
            "Alexandria" "Algiers" "Alice Springs" "Almaty" "Amsterdam" "Anadyr" "Anchorage" "Andorra la Vella" "Ankara"
            "Antananarivo" "Antsiranana" "Arkhangelsk" "Ashgabat" "Asmara" "Assab" "Astana" "Athens" "Atlanta" "Auckland"
            "Austin" "Baghdad" "Baguio" "Baku" "Baltimore" "Bamako" "Bangkok" "Bangui" "Banjul" "Barcelona" "Bata"
            "Batumi" "Beijing" "Beirut" "Belgrade" "Belize City" "Benghazi" "Bergen" "Berlin" "Bilbao" "Birao" "Bishkek"
            "Bissau" "Blantyre" "Bloemfontein" "Boise" "Bordeaux" "Bosaso" "Boston" "Bouaké" "Bratislava" "Brazzaville"
            "Bridgetown" "Brisbane" "Brussels" "Bucharest" "Budapest" "Bujumbura" "Bulawayo" "Burnie" "Busan"
            "Cabo San Lucas" "Cairns" "Cairo" "Calgary" "Canberra" "Cape Town" "Changsha" "Charlotte" "Chiang Mai"
            "Chicago" "Chihuahua" "Chișinău" "Chittagong" "Chongqing" "Christchurch" "City of San Marino" "Colombo"
            "Columbus" "Conakry" "Copenhagen" "Cotonou" "Cracow" "Da Lat" "Da Nang" "Dakar" "Dallas" "Damascus"
            "Dampier" "Dar es Salaam" "Darwin" "Denpasar" "Denver" "Detroit" "Dhaka" "Dikson" "Dili" "Djibouti"
            "Dodoma" "Dolisie" "Douala" "Dubai" "Dublin" "Dunedin" "Durban" "Dushanbe" "Edinburgh" "Edmonton" "El Paso"
            "Entebbe" "Erbil" "Erzurum" "Fairbanks" "Fianarantsoa" "Flores, Petén" "Frankfurt" "Fresno" "Fukuoka" "Gabès"
            "Gaborone" "Gagnoa" "Gangtok" "Garissa" "Garoua" "George Town" "Ghanzi" "Gjoa Haven" "Guadalajara" "Guangzhou"
            "Guatemala City" "Halifax" "Hamburg" "Hamilton" "Hanga Roa" "Hanoi" "Harare" "Harbin" "Hargeisa" "Hat Yai"
            "Havana" "Helsinki" "Heraklion" "Hiroshima" "Ho Chi Minh City" "Hobart" "Hong Kong" "Honiara" "Honolulu"
            "Houston" "Ifrane" "Indianapolis" "Iqaluit" "Irkutsk" "Istanbul" "İzmir" "Jacksonville" "Jakarta" "Jayapura"
            "Jerusalem" "Johannesburg" "Jos" "Juba" "Kabul" "Kampala" "Kandi" "Kankan" "Kano" "Kansas City" "Karachi"
            "Karonga" "Kathmandu" "Khartoum" "Kingston" "Kinshasa" "Kolkata" "Kuala Lumpur" "Kumasi" "Kunming" "Kuopio"
            "Kuwait City" "Kyiv" "Kyoto" "La Ceiba" "La Paz" "Lagos" "Lahore" "Lake Havasu City" "Lake Tekapo"
            "Las Palmas de Gran Canaria" "Las Vegas" "Launceston" "Lhasa" "Libreville" "Lisbon" "Livingstone" "Ljubljana"
            "Lomé" "London" "Los Angeles" "Louisville" "Luanda" "Lubumbashi" "Lusaka" "Luxembourg City" "Lviv" "Macapá"
            "Maceió" "Machala" "Madrid" "Málaga" "Malindi" "Managua" "Manaus" "Manchester" "Manila" "Maputo" "Maracaibo"
            "Marrakech" "Marseille" "Maseru" "Masindi" "Matadi" "Mazatlán" "Mbabane" "Mbarara" "McMurdo Station"
            "Medan" "Medellín" "Melbourne" "Mendoza" "Mexico City" "Miami" "Milan" "Minsk" "Monaco" "Monrovia" "Montevideo"
            "Monterrey" "Montréal" "Moose Factory" "Mopti" "Moroni" "Moscow" "Mossel Bay" "Mumbai" "Munich" "Nairobi"
            "Nakhchivan" "Nairobi" "Nakhchivan" "Nakhodka" "Nalut" "Nampula" "Nanjing" "Nanning" "Nantes" "Naples"
            "Naypyidaw" "Ndola" "N'Djamena" "Nelson" "New Delhi" "New York City" "Niamey" "Nicosia" "Nizhnevartovsk"
            "Nizhny Novgorod" "Nomuka" "Norilsk" "Nouakchott" "Nouméa" "Nukualofa" "Odessa" "Omsk" "Oranjestad" "Osaka"
            "Oslo" "Ottawa" "Padang" "Palikir" "Panama City" "Papeete" "Paramaribo" "Paris" "Pemba" "Perth" "Peshawar"
            "Phnom Penh" "Phoenix" "Pointe-à-Pitre" "Ponta Delgada" "Port Harcourt" "Port Louis" "Port Moresby"
            "Port of Spain" "Port Vila" "Portland" "Porto" "Porto-Novo" "Prague" "Praia" "Pretoria" "Providence"
            "Puebla" "Puerto Baquerizo Moreno" "Puerto Montt" "Puerto Princesa" "Punta Arenas" "Pyongyang" "Quebec City"
            "Queenstown" "Quito" "Rabat" "Recife" "Reykjavik" "Richmond" "Rio de Janeiro" "Riyadh" "Rome" "Rostov-on-Don"
            "Rotterdam" "Sacramento" "Saint-Denis" "Saint George's" "Saint Helier" "Saint John's" "Saint Peter Port"
            "Saint Pierre" "Saint-Denis" "Saint-Pierre" "Salzburg" "Samara" "San Antonio" "San Diego" "San Francisco"
            "San José" "San Juan" "San Luis Potosí" "San Marino" "San Miguel de Tucumán" "San Pedro Sula" "San Salvador"
            "Santa Cruz de la Sierra" "Santa Fe" "Santiago" "Santo Domingo" "São Luís" "São Paulo" "Sapporo" "Sarajevo"
            "Saratov" "Seattle" "Sejong City" "Semarang" "Seoul" "Sergiyev Posad" "Sevastopol" "Shanghai" "Shantou"
            "Shenyang" "Shenzhen" "Sherbrooke" "Shiraz" "Sibolga" "Siem Reap" "Simferopol" "Singapore" "Skopje" "Sofia"
            "Songkhla" "Sorong" "Sørkapp Land" "South Tarawa" "Split" "Sri Jayawardenepura Kotte" "St. John's" "St. Louis"
            "St. Petersburg" "Stanley" "Stockholm" "Sucre" "Suez" "Suva" "Sydney" "Sylvan Lake" "Tabriz" "Taichung"
            "Tainan" "Taipei" "Tallahassee" "Tallinn" "Tampa" "Tangier" "Tashkent" "Tbilisi" "Tegucigalpa" "Tehran"
            "Tel Aviv" "Thimphu" "Tianjin" "Tijuana" "Timișoara" "Tiraspol" "Tokyo" "Toronto" "Tórshavn" "Toulon"
            "Trondheim" "Tunis" "Turin" "Ulaanbaatar" "Ulan-Ude" "Ulanhot" "Umeå" "Ushuaia" "Utrecht" "Vaduz" "Valdivia"
            "Valletta" "Vancouver" "Vatican City" "Velas" "Venice" "Victoria" "Vienna" "Vientiane" "Vigo" "Vilnius"
            "Virginia Beach" "Vladikavkaz" "Vladimir" "Vladivostok" "Vlorë" "Volgograd" "Vologda" "Voronezh" "Wa"
            "Warsaw" "Washington, D.C." "Wellington" "Wichita" "Windhoek" "Winnipeg" "Wroclaw" "Wuhan" "Wuzhou"
            "Xiamen" "Xi'an" "Yakutsk" "Yalta" "Yamoussoukro" "Yan'an" "Yaroslavl" "Yerevan" "Yinchuan" "Yokohama"
            "Yoshkar-Ola" "Yuzhno-Sakhalinsk" "Zagreb" "Zamboanga City" "Zanzibar City" "Zaporizhzhia" "Zaragoza"
            "Zarqa" "Zhengzhou" "Zürich" "Zvishavane")
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
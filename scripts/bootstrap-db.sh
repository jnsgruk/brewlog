#!/usr/bin/env bash

cargo build

if [[ -z "$BREWLOG_TOKEN" ]]; then
  echo "Error: BREWLOG_TOKEN environment variable is not set."
  exit 1
fi

# Tim Wendelboe (Norway)
./target/debug/brewlog add-roaster \
  --name "Tim Wendelboe" \
  --country "Norway" \
  --city "Oslo" \
  --homepage "https://timwendelboe.no" \
  --notes "World-renowned Nordic micro-roastery dedicated to clarity and sustainability."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Tim Wendelboe") | .id')" \
  --name "Ben Saïd Natural" \
  --origin "Ethiopia" \
  --region "Sidamo" \
  --producer "Ben Saïd" \
  --process "Natural" \
  --tasting-notes "Bergamot, Apricot, Floral"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Tim Wendelboe") | .id')" \
  --name "Finca Tamana Washed" \
  --origin "Colombia" \
  --region "El Pital, Huila" \
  --producer "Elias Roa" \
  --process "Washed" \
  --tasting-notes "Red Apple, Vanilla, Caramel"


# Coffee Collective (Denmark)
./target/debug/brewlog add-roaster \
  --name "Coffee Collective" \
  --country "Denmark" \
  --city "Copenhagen" \
  --homepage "https://coffeecollective.dk" \
  --notes "Pioneers of transparency and sustainability; multi-time Nordic roaster award winners."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Coffee Collective") | .id')" \
  --name "Daterra Sweet Collection" \
  --origin "Brazil" \
  --region "Cerrado" \
  --producer "Daterra" \
  --process "Pulped Natural" \
  --tasting-notes "Hazelnut, Milk Chocolate, Yellow Fruit"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Coffee Collective") | .id')" \
  --name "Kieni" \
  --origin "Kenya" \
  --region "Nyeri" \
  --producer "Kieni Factory" \
  --process "Washed" \
  --tasting-notes "Currant, Black Tea, Grape"


# Drop Coffee (Sweden)
./target/debug/brewlog add-roaster \
  --name "Drop Coffee" \
  --country "Sweden" \
  --city "Stockholm" \
  --homepage "https://dropcoffee.com" \
  --notes "Award-winning Swedish roastery prized for its elegance and clean Scandinavian style."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Drop Coffee") | .id')" \
  --name "La Linda" \
  --origin "Bolivia" \
  --region "Caranavi" \
  --producer "Pedro Rodriguez" \
  --process "Washed" \
  --tasting-notes "Red Apple, Caramel, Floral"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Drop Coffee") | .id')" \
  --name "El Sunzita" \
  --origin "El Salvador" \
  --region "Ahuachapan" \
  --producer "Jorge Raul Rivera" \
  --process "Natural" \
  --tasting-notes "Strawberry, Mango, Dark Chocolate"


# La Cabra (Denmark)
./target/debug/brewlog add-roaster \
  --name "La Cabra" \
  --country "Denmark" \
  --city "Aarhus" \
  --homepage "https://www.lacabra.dk" \
  --notes "Scandinavian minimalist roastery known for clarity and innovative sourcing."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="La Cabra") | .id')" \
  --name "Halo Beriti" \
  --origin "Ethiopia" \
  --region "Yirgacheffe" \
  --producer "Halo Beriti Cooperative" \
  --process "Washed" \
  --tasting-notes "Jasmine, Lemon, Stone Fruit"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="La Cabra") | .id')" \
  --name "Cerro Azul" \
  --origin "Colombia" \
  --region "Valle del Cauca" \
  --producer "Granja La Esperanza" \
  --process "Washed" \
  --tasting-notes "Blueberry, Plum, Grapefruit"


# April Coffee (Denmark)
./target/debug/brewlog add-roaster \
  --name "April Coffee" \
  --country "Denmark" \
  --city "Copenhagen" \
  --homepage "https://aprilcoffeeroasters.com" \
  --notes "Modern approach to Nordic coffee, emphasizing transparency and traceability."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="April Coffee") | .id')" \
  --name "El Salvador Pacamara" \
  --origin "El Salvador" \
  --region "Santa Ana" \
  --producer "Ernesto Menendez" \
  --process "Honey" \
  --tasting-notes "Grapefruit, Sugar Cane, Plum"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="April Coffee") | .id')" \
  --name "Guji Highland" \
  --origin "Ethiopia" \
  --region "Guji" \
  --producer "Andualem Abebe" \
  --process "Natural" \
  --tasting-notes "Peach, Strawberry, Cream"


# Assembly Coffee (UK)
./target/debug/brewlog add-roaster \
  --name "Assembly Coffee" \
  --country "UK" \
  --city "London" \
  --homepage "https://assemblycoffee.co.uk" \
  --notes "Based in Brixton, Assembly focuses on collaborative sourcing and education."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Assembly Coffee") | .id')" \
  --name "Kochere" \
  --origin "Ethiopia" \
  --region "Yirgacheffe" \
  --producer "Kochere Region Growers" \
  --process "Washed" \
  --tasting-notes "Peach, Lemon, Jasmine"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Assembly Coffee") | .id')" \
  --name "La Laja" \
  --origin "Mexico" \
  --region "Veracruz" \
  --producer "La Laja Estate" \
  --process "Natural" \
  --tasting-notes "Cherry, Milk Chocolate, Praline"


# Square Mile (UK)
./target/debug/brewlog add-roaster \
  --name "Square Mile Coffee" \
  --country "UK" \
  --city "London" \
  --homepage "https://squaremilecoffee.com" \
  --notes "One of London's pioneers; delivers balanced and clear, fruit-forward coffees."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Square Mile Coffee") | .id')" \
  --name "Red Brick Espresso" \
  --origin "Blend" \
  --region "Multiple Origins" \
  --producer "Various" \
  --process "Washed, Natural" \
  --tasting-notes "Berry, Chocolate, Citrus"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Square Mile Coffee") | .id')" \
  --name "Kamwangi" \
  --origin "Kenya" \
  --region "Kirinyaga" \
  --producer "Kamwangi Factory" \
  --process "Washed" \
  --tasting-notes "Blackcurrant, Rhubarb, Blood Orange"


# Dak Coffee Roasters (Netherlands)
./target/debug/brewlog add-roaster \
  --name "Dak Coffee Roasters" \
  --country "Netherlands" \
  --city "Amsterdam" \
  --homepage "https://www.dakcoffeeroasters.com" \
  --notes "Highly experimental Dutch roastery; celebrates vibrant acidity and alternative processing."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Dak Coffee Roasters") | .id')" \
  --name "El Paraiso 92 Anaerobic" \
  --origin "Colombia" \
  --region "Cauca" \
  --producer "Diego Bermudez" \
  --process "Thermal Shock Anaerobic" \
  --tasting-notes "Passionfruit, Raspberry, Yogurt"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Dak Coffee Roasters") | .id')" \
  --name "Oreti SL28" \
  --origin "Kenya" \
  --region "Kirinyaga" \
  --producer "Oreti Estate" \
  --process "Washed" \
  --tasting-notes "Grapefruit, Blackcurrant, Plum"


# Bonanza Coffee (Germany)
./target/debug/brewlog add-roaster \
  --name "Bonanza Coffee" \
  --country "Germany" \
  --city "Berlin" \
  --homepage "https://www.bonanzacoffee.de" \
  --notes "Pioneering Berlin roastery focused on brightness, balance, and freshness."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Bonanza Coffee") | .id')" \
  --name "Gatomboya" \
  --origin "Kenya" \
  --region "Nyeri" \
  --producer "Gatomboya Cooperative" \
  --process "Washed" \
  --tasting-notes "Blackcurrant, Lime, Tomato"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Bonanza Coffee") | .id')" \
  --name "Los Pirineos" \
  --origin "El Salvador" \
  --region "Usulután" \
  --producer "Gilberto Baraona" \
  --process "Honey" \
  --tasting-notes "Maple, Fudge, Green Apple"


# Friedhats (Netherlands)
./target/debug/brewlog add-roaster \
  --name "Friedhats" \
  --country "Netherlands" \
  --city "Amsterdam" \
  --homepage "https://friedhats.com" \
  --notes "Quirky branding meets serious, awarded, fruit-forward coffees from Amsterdam."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Friedhats") | .id')" \
  --name "Sidamo Guji" \
  --origin "Ethiopia" \
  --region "Guji" \
  --producer "Smallholders" \
  --process "Natural" \
  --tasting-notes "Peach, Raspberry, Rosehip"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Friedhats") | .id')" \
  --name "La Esmeralda Geisha" \
  --origin "Panama" \
  --region "Boquete" \
  --producer "Hacienda La Esmeralda" \
  --process "Washed" \
  --tasting-notes "Jasmine, Bergamot, Papaya"


# Origin Coffee (UK)
./target/debug/brewlog add-roaster \
  --name "Origin Coffee" \
  --country "UK" \
  --city "Porthleven" \
  --homepage "https://origincoffee.co.uk" \
  --notes "Specialty roaster with close partnerships at origin; leading UK scene with cutting-edge lots."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Origin Coffee") | .id')" \
  --name "San Fermin" \
  --origin "Colombia" \
  --region "Tolima" \
  --producer "San Fermin Smallholders" \
  --process "Washed" \
  --tasting-notes "Red Grape, Caramel, Blood Orange"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Origin Coffee") | .id')" \
  --name "Aricha" \
  --origin "Ethiopia" \
  --region "Yirgacheffe" \
  --producer "Aricha Washing Station" \
  --process "Washed" \
  --tasting-notes "Honey, Peach, Black Tea"


# Dark Arts Coffee (UK)
./target/debug/brewlog add-roaster \
  --name "Dark Arts Coffee" \
  --country "UK" \
  --city "London" \
  --homepage "https://www.darkartscoffee.co.uk" \
  --notes "Playful, disruptive roaster with a cult following and flavor-forward offerings."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Dark Arts Coffee") | .id')" \
  --name "Death to Decaf" \
  --origin "Brazil" \
  --region "Minas Gerais" \
  --producer "Carmo de Minas" \
  --process "Swiss Water Decaf" \
  --tasting-notes "Cocoa, Cherry, Almond"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Dark Arts Coffee") | .id')" \
  --name "Snoop" \
  --origin "Guatemala" \
  --region "Huehuetenango" \
  --producer "Various Smallholders" \
  --process "Washed" \
  --tasting-notes "Toffee, Green Apple, Plum"


# KAWA Coffee (France)
./target/debug/brewlog add-roaster \
  --name "KAWA Coffee" \
  --country "France" \
  --city "Paris" \
  --homepage "https://www.kawa.coffee" \
  --notes "One of Paris’ most exciting specialty roasteries, known for unusual and competition-level lots."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="KAWA Coffee") | .id')" \
  --name "Sudan Rume" \
  --origin "Colombia" \
  --region "Cauca" \
  --producer "Granja La Esperanza" \
  --process "Natural" \
  --tasting-notes "Strawberry, Cinnamon, Grape"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="KAWA Coffee") | .id')" \
  --name "Arbegona" \
  --origin "Ethiopia" \
  --region "Sidama" \
  --producer "Arbegona Washing Station" \
  --process "Washed" \
  --tasting-notes "Violet, Apricot, Lemon"


# Stow Coffee (Slovenia)
./target/debug/brewlog add-roaster \
  --name "Stow Coffee" \
  --country "Slovenia" \
  --city "Ljubljana" \
  --homepage "https://www.stowcoffee.com" \
  --notes "Slovenia’s specialty leader, awarded for pure, brightly acidic profiles and innovation."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Stow Coffee") | .id')" \
  --name "Santa Barbara" \
  --origin "Honduras" \
  --region "Santa Barbara" \
  --producer "Benjamin Paz" \
  --process "Honey" \
  --tasting-notes "Red Currant, Honeydew, Cocoa"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Stow Coffee") | .id')" \
  --name "Suke Quto" \
  --origin "Ethiopia" \
  --region "Guji" \
  --producer "Tesfaye Bekele" \
  --process "Natural" \
  --tasting-notes "Blackberry, Vanilla, Jasmine"


# Bows Coffee (Canada)
./target/debug/brewlog add-roaster \
  --name "Bows Coffee" \
  --country "Canada" \
  --city "Victoria" \
  --homepage "https://bowscoffee.com" \
  --notes "Canadian micro-roaster with focus on clarity, complexity, and ethical sourcing."

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Bows Coffee") | .id')" \
  --name "La Chumeca" \
  --origin "Costa Rica" \
  --region "Tarrazú" \
  --producer "Doña Olga Jiménez" \
  --process "White Honey" \
  --tasting-notes "Mandarin, Honeycomb, Almond"

./target/debug/brewlog add-roast \
  --roaster-id "$(./target/debug/brewlog list-roasters | jq -r '.[] | select(.name=="Bows Coffee") | .id')" \
  --name "Simbi" \
  --origin "Rwanda" \
  --region "Huye" \
  --producer "Simbi Co-op" \
  --process "Washed" \
  --tasting-notes "Black Tea, Orange, Cane Sugar"

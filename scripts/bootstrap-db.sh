#!/usr/bin/env bash
set -euo pipefail

cargo build

# BREWLOG_TOKEN must be set before running this script.
# To create a token:
#   1. Start the server: ./target/debug/brewlog serve
#   2. Register at the URL printed on first start
#   3. Create a token: ./target/debug/brewlog token create --name "bootstrap-token"
#   4. Export the token: export BREWLOG_TOKEN=<token>
if [[ -z "${BREWLOG_TOKEN:-}" ]]; then
  echo "Error: BREWLOG_TOKEN environment variable is not set."
  echo "Create a token first: ./target/debug/brewlog token create --name bootstrap-token"
  exit 1
fi

# Tim Wendelboe (Norway)
./target/debug/brewlog roaster add \
  --name "Tim Wendelboe" \
  --country "Norway" \
  --city "Oslo" \
  --homepage "https://timwendelboe.no"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Tim Wendelboe") | .id')" \
  --name "Ben Saïd Natural" \
  --origin "Ethiopia" \
  --region "Sidamo" \
  --producer "Ben Saïd" \
  --process "Natural" \
  --tasting-notes "Bergamot, Apricot, Floral"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Tim Wendelboe") | .id')" \
  --name "Finca Tamana Washed" \
  --origin "Colombia" \
  --region "El Pital, Huila" \
  --producer "Elias Roa" \
  --process "Washed" \
  --tasting-notes "Red Apple, Vanilla, Caramel"


# Coffee Collective (Denmark)
./target/debug/brewlog roaster add \
  --name "Coffee Collective" \
  --country "Denmark" \
  --city "Copenhagen" \
  --homepage "https://coffeecollective.dk"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Coffee Collective") | .id')" \
  --name "Daterra Sweet Collection" \
  --origin "Brazil" \
  --region "Cerrado" \
  --producer "Daterra" \
  --process "Pulped Natural" \
  --tasting-notes "Hazelnut, Milk Chocolate, Yellow Fruit"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Coffee Collective") | .id')" \
  --name "Kieni" \
  --origin "Kenya" \
  --region "Nyeri" \
  --producer "Kieni Factory" \
  --process "Washed" \
  --tasting-notes "Currant, Black Tea, Grape"


# Drop Coffee (Sweden)
./target/debug/brewlog roaster add \
  --name "Drop Coffee" \
  --country "Sweden" \
  --city "Stockholm" \
  --homepage "https://dropcoffee.com"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Drop Coffee") | .id')" \
  --name "La Linda" \
  --origin "Bolivia" \
  --region "Caranavi" \
  --producer "Pedro Rodriguez" \
  --process "Washed" \
  --tasting-notes "Red Apple, Caramel, Floral"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Drop Coffee") | .id')" \
  --name "El Sunzita" \
  --origin "El Salvador" \
  --region "Ahuachapan" \
  --producer "Jorge Raul Rivera" \
  --process "Natural" \
  --tasting-notes "Strawberry, Mango, Dark Chocolate"


# La Cabra (Denmark)
./target/debug/brewlog roaster add \
  --name "La Cabra" \
  --country "Denmark" \
  --city "Aarhus" \
  --homepage "https://www.lacabra.dk"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="La Cabra") | .id')" \
  --name "Halo Beriti" \
  --origin "Ethiopia" \
  --region "Yirgacheffe" \
  --producer "Halo Beriti Cooperative" \
  --process "Washed" \
  --tasting-notes "Jasmine, Lemon, Stone Fruit"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="La Cabra") | .id')" \
  --name "Cerro Azul" \
  --origin "Colombia" \
  --region "Valle del Cauca" \
  --producer "Granja La Esperanza" \
  --process "Washed" \
  --tasting-notes "Blueberry, Plum, Grapefruit"


# April Coffee (Denmark)
./target/debug/brewlog roaster add \
  --name "April Coffee" \
  --country "Denmark" \
  --city "Copenhagen" \
  --homepage "https://aprilcoffeeroasters.com"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="April Coffee") | .id')" \
  --name "El Salvador Pacamara" \
  --origin "El Salvador" \
  --region "Santa Ana" \
  --producer "Ernesto Menendez" \
  --process "Honey" \
  --tasting-notes "Grapefruit, Sugar Cane, Plum"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="April Coffee") | .id')" \
  --name "Guji Highland" \
  --origin "Ethiopia" \
  --region "Guji" \
  --producer "Andualem Abebe" \
  --process "Natural" \
  --tasting-notes "Peach, Strawberry, Cream"


# Assembly Coffee (UK)
./target/debug/brewlog roaster add \
  --name "Assembly Coffee" \
  --country "United Kingdom" \
  --city "London" \
  --homepage "https://assemblycoffee.co.uk"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Assembly Coffee") | .id')" \
  --name "Kochere" \
  --origin "Ethiopia" \
  --region "Yirgacheffe" \
  --producer "Kochere Region Growers" \
  --process "Washed" \
  --tasting-notes "Peach, Lemon, Jasmine"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Assembly Coffee") | .id')" \
  --name "La Laja" \
  --origin "Mexico" \
  --region "Veracruz" \
  --producer "La Laja Estate" \
  --process "Natural" \
  --tasting-notes "Cherry, Milk Chocolate, Praline"


# Square Mile (UK)
./target/debug/brewlog roaster add \
  --name "Square Mile Coffee" \
  --country "United Kingdom" \
  --city "London" \
  --homepage "https://squaremilecoffee.com"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Square Mile Coffee") | .id')" \
  --name "Red Brick Espresso" \
  --origin "Blend" \
  --region "Multiple Origins" \
  --producer "Various" \
  --process "Washed, Natural" \
  --tasting-notes "Berry, Chocolate, Citrus"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Square Mile Coffee") | .id')" \
  --name "Kamwangi" \
  --origin "Kenya" \
  --region "Kirinyaga" \
  --producer "Kamwangi Factory" \
  --process "Washed" \
  --tasting-notes "Blackcurrant, Rhubarb, Blood Orange"


# Dak Coffee Roasters (Netherlands)
./target/debug/brewlog roaster add \
  --name "Dak Coffee Roasters" \
  --country "Netherlands" \
  --city "Amsterdam" \
  --homepage "https://www.dakcoffeeroasters.com"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Dak Coffee Roasters") | .id')" \
  --name "El Paraiso 92 Anaerobic" \
  --origin "Colombia" \
  --region "Cauca" \
  --producer "Diego Bermudez" \
  --process "Thermal Shock Anaerobic" \
  --tasting-notes "Passionfruit, Raspberry, Yogurt"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Dak Coffee Roasters") | .id')" \
  --name "Oreti SL28" \
  --origin "Kenya" \
  --region "Kirinyaga" \
  --producer "Oreti Estate" \
  --process "Washed" \
  --tasting-notes "Grapefruit, Blackcurrant, Plum"


# Bonanza Coffee (Germany)
./target/debug/brewlog roaster add \
  --name "Bonanza Coffee" \
  --country "Germany" \
  --city "Berlin" \
  --homepage "https://www.bonanzacoffee.de"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Bonanza Coffee") | .id')" \
  --name "Gatomboya" \
  --origin "Kenya" \
  --region "Nyeri" \
  --producer "Gatomboya Cooperative" \
  --process "Washed" \
  --tasting-notes "Blackcurrant, Lime, Tomato"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Bonanza Coffee") | .id')" \
  --name "Los Pirineos" \
  --origin "El Salvador" \
  --region "Usulután" \
  --producer "Gilberto Baraona" \
  --process "Honey" \
  --tasting-notes "Maple, Fudge, Green Apple"


# Friedhats (Netherlands)
./target/debug/brewlog roaster add \
  --name "Friedhats" \
  --country "Netherlands" \
  --city "Amsterdam" \
  --homepage "https://friedhats.com"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Friedhats") | .id')" \
  --name "Sidamo Guji" \
  --origin "Ethiopia" \
  --region "Guji" \
  --producer "Smallholders" \
  --process "Natural" \
  --tasting-notes "Peach, Raspberry, Rosehip"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Friedhats") | .id')" \
  --name "La Esmeralda Geisha" \
  --origin "Panama" \
  --region "Boquete" \
  --producer "Hacienda La Esmeralda" \
  --process "Washed" \
  --tasting-notes "Jasmine, Bergamot, Papaya"


# Origin Coffee (UK)
./target/debug/brewlog roaster add \
  --name "Origin Coffee" \
  --country "United Kingdom" \
  --city "Porthleven" \
  --homepage "https://origincoffee.co.uk"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Origin Coffee") | .id')" \
  --name "San Fermin" \
  --origin "Colombia" \
  --region "Tolima" \
  --producer "San Fermin Smallholders" \
  --process "Washed" \
  --tasting-notes "Red Grape, Caramel, Blood Orange"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Origin Coffee") | .id')" \
  --name "Aricha" \
  --origin "Ethiopia" \
  --region "Yirgacheffe" \
  --producer "Aricha Washing Station" \
  --process "Washed" \
  --tasting-notes "Honey, Peach, Black Tea"


# Dark Arts Coffee (UK)
./target/debug/brewlog roaster add \
  --name "Dark Arts Coffee" \
  --country "United Kingdom" \
  --city "London" \
  --homepage "https://www.darkartscoffee.co.uk"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Dark Arts Coffee") | .id')" \
  --name "Death to Decaf" \
  --origin "Brazil" \
  --region "Minas Gerais" \
  --producer "Carmo de Minas" \
  --process "Swiss Water Decaf" \
  --tasting-notes "Cocoa, Cherry, Almond"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Dark Arts Coffee") | .id')" \
  --name "Snoop" \
  --origin "Guatemala" \
  --region "Huehuetenango" \
  --producer "Various Smallholders" \
  --process "Washed" \
  --tasting-notes "Toffee, Green Apple, Plum"


# KAWA Coffee (France)
./target/debug/brewlog roaster add \
  --name "KAWA Coffee" \
  --country "France" \
  --city "Paris" \
  --homepage "https://www.kawa.coffee"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="KAWA Coffee") | .id')" \
  --name "Sudan Rume" \
  --origin "Colombia" \
  --region "Cauca" \
  --producer "Granja La Esperanza" \
  --process "Natural" \
  --tasting-notes "Strawberry, Cinnamon, Grape"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="KAWA Coffee") | .id')" \
  --name "Arbegona" \
  --origin "Ethiopia" \
  --region "Sidama" \
  --producer "Arbegona Washing Station" \
  --process "Washed" \
  --tasting-notes "Violet, Apricot, Lemon"


# Stow Coffee (Slovenia)
./target/debug/brewlog roaster add \
  --name "Stow Coffee" \
  --country "Slovenia" \
  --city "Ljubljana" \
  --homepage "https://www.stowcoffee.com"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Stow Coffee") | .id')" \
  --name "Santa Barbara" \
  --origin "Honduras" \
  --region "Santa Barbara" \
  --producer "Benjamin Paz" \
  --process "Honey" \
  --tasting-notes "Red Currant, Honeydew, Cocoa"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Stow Coffee") | .id')" \
  --name "Suke Quto" \
  --origin "Ethiopia" \
  --region "Guji" \
  --producer "Tesfaye Bekele" \
  --process "Natural" \
  --tasting-notes "Blackberry, Vanilla, Jasmine"


# Bows Coffee (Canada)
./target/debug/brewlog roaster add \
  --name "Bows Coffee" \
  --country "Canada" \
  --city "Victoria" \
  --homepage "https://bowscoffee.com"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Bows Coffee") | .id')" \
  --name "La Chumeca" \
  --origin "Costa Rica" \
  --region "Tarrazú" \
  --producer "Doña Olga Jiménez" \
  --process "White Honey" \
  --tasting-notes "Mandarin, Honeycomb, Almond"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Bows Coffee") | .id')" \
  --name "Simbi" \
  --origin "Rwanda" \
  --region "Huye" \
  --producer "Simbi Co-op" \
  --process "Washed" \
  --tasting-notes "Black Tea, Orange, Cane Sugar"

# ============================================================================
# Cafes - 10 cafes across London, Madrid, Berlin, Munich, and Bristol
# ============================================================================

# London - Monmouth Coffee (Borough Market)
./target/debug/brewlog cafe add \
  --name "Monmouth Coffee" \
  --city "London" \
  --country "United Kingdom" \
  --latitude 51.5055 \
  --longitude -0.0910 \
  --website "https://www.monmouthcoffee.co.uk"

# London - Prufrock Coffee
./target/debug/brewlog cafe add \
  --name "Prufrock Coffee" \
  --city "London" \
  --country "United Kingdom" \
  --latitude 51.5246 \
  --longitude -0.1098 \
  --website "https://www.prufrockcoffee.com"

# Madrid - Hola Coffee
./target/debug/brewlog cafe add \
  --name "Hola Coffee" \
  --city "Madrid" \
  --country "Spain" \
  --latitude 40.4285 \
  --longitude -3.7025 \
  --website "https://www.holacoffee.es"

# Madrid - Toma Café
./target/debug/brewlog cafe add \
  --name "Toma Café" \
  --city "Madrid" \
  --country "Spain" \
  --latitude 40.4260 \
  --longitude -3.7075

# Berlin - The Barn
./target/debug/brewlog cafe add \
  --name "The Barn" \
  --city "Berlin" \
  --country "Germany" \
  --latitude 52.5298 \
  --longitude 13.4020 \
  --website "https://thebarn.de"

# Berlin - Companion Coffee
./target/debug/brewlog cafe add \
  --name "Companion Coffee" \
  --city "Berlin" \
  --country "Germany" \
  --latitude 52.4952 \
  --longitude 13.4188 \
  --website "https://www.companion.coffee"

# Munich - Man Versus Machine
./target/debug/brewlog cafe add \
  --name "Man Versus Machine" \
  --city "Munich" \
  --country "Germany" \
  --latitude 48.1310 \
  --longitude 11.5690 \
  --website "https://www.mvsmcoffee.de"

# Munich - Vits der Kaffee
./target/debug/brewlog cafe add \
  --name "Vits der Kaffee" \
  --city "Munich" \
  --country "Germany" \
  --latitude 48.1353 \
  --longitude 11.5741 \
  --website "https://www.vfrischekaffee.de"

# Bristol - Full Court Press
./target/debug/brewlog cafe add \
  --name "Full Court Press" \
  --city "Bristol" \
  --country "United Kingdom" \
  --latitude 51.4543 \
  --longitude -2.5930 \
  --website "https://www.fullcourtpress.coffee"

# Bristol - Small Street Espresso
./target/debug/brewlog cafe add \
  --name "Small Street Espresso" \
  --city "Bristol" \
  --country "United Kingdom" \
  --latitude 51.4540 \
  --longitude -2.5955

# ============================================================================
# Cups - 1 to 3 cups per cafe, pairing roasts with cafes visited
# ============================================================================

# Monmouth Coffee (London) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Red Brick Espresso") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Monmouth Coffee") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Kamwangi") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Monmouth Coffee") | .id')"

# Prufrock Coffee (London) - 3 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Kochere") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Prufrock Coffee") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="La Laja") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Prufrock Coffee") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Ben Saïd Natural") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Prufrock Coffee") | .id')"

# Hola Coffee (Madrid) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Cerro Azul") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Hola Coffee") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Guji Highland") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Hola Coffee") | .id')"

# Toma Café (Madrid) - 1 cup
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Daterra Sweet Collection") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Toma Café") | .id')"

# The Barn (Berlin) - 3 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Gatomboya") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="The Barn") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Los Pirineos") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="The Barn") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Halo Beriti") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="The Barn") | .id')"

# Companion Coffee (Berlin) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="El Paraiso 92 Anaerobic") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Companion Coffee") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Oreti SL28") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Companion Coffee") | .id')"

# Man Versus Machine (Munich) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Finca Tamana Washed") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Man Versus Machine") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="La Esmeralda Geisha") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Man Versus Machine") | .id')"

# Vits der Kaffee (Munich) - 1 cup
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="San Fermin") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Vits der Kaffee") | .id')"

# Full Court Press (Bristol) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Suke Quto") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Full Court Press") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Death to Decaf") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Full Court Press") | .id')"

# Small Street Espresso (Bristol) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Simbi") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Small Street Espresso") | .id')"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="La Chumeca") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Small Street Espresso") | .id')"

# ============================================================================
# Bags - Various bags from different roasters with amounts ranging 100g-500g
# ============================================================================

# Tim Wendelboe - Ben Saïd Natural (250g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Ben Saïd Natural") | .id')" \
  --roast-date "2026-01-15" \
  --amount 250

# Tim Wendelboe - Finca Tamana Washed (350g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Finca Tamana Washed") | .id')" \
  --roast-date "2026-01-18" \
  --amount 350

# Coffee Collective - Daterra Sweet Collection (200g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Daterra Sweet Collection") | .id')" \
  --roast-date "2026-01-10" \
  --amount 200

# Drop Coffee - La Linda (500g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="La Linda") | .id')" \
  --roast-date "2026-01-20" \
  --amount 500

# La Cabra - Halo Beriti (150g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Halo Beriti") | .id')" \
  --roast-date "2026-01-12" \
  --amount 150

# April Coffee - Guji Highland (300g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Guji Highland") | .id')" \
  --roast-date "2026-01-22" \
  --amount 300

# Assembly Coffee - Kochere (250g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Kochere") | .id')" \
  --roast-date "2026-01-08" \
  --amount 250

# Square Mile Coffee - Red Brick Espresso (400g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Red Brick Espresso") | .id')" \
  --roast-date "2026-01-25" \
  --amount 400

# Dak Coffee Roasters - El Paraiso 92 Anaerobic (100g - small competition lot)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="El Paraiso 92 Anaerobic") | .id')" \
  --roast-date "2026-01-28" \
  --amount 100

# Bonanza Coffee - Gatomboya (175g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Gatomboya") | .id')" \
  --roast-date "2026-01-05" \
  --amount 175

# Stow Coffee - Suke Quto (225g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Suke Quto") | .id')" \
  --roast-date "2026-01-30" \
  --amount 225

# Bows Coffee - Simbi (450g)
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Simbi") | .id')" \
  --roast-date "2026-01-14" \
  --amount 450

# ============================================================================
# Finished Bags - Mark 4 older bags as finished
# ============================================================================

# Finish Gatomboya bag (oldest - Jan 5)
./target/debug/brewlog bag update \
  --id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Gatomboya") | .id')" \
  --closed true \
  --finished-at "2026-01-20"

# Finish Kochere bag (Jan 8)
./target/debug/brewlog bag update \
  --id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Kochere") | .id')" \
  --closed true \
  --finished-at "2026-01-22"

# Finish Daterra Sweet Collection bag (Jan 10)
./target/debug/brewlog bag update \
  --id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Daterra Sweet Collection") | .id')" \
  --closed true \
  --finished-at "2026-01-25"

# Finish Halo Beriti bag (Jan 12)
./target/debug/brewlog bag update \
  --id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Halo Beriti") | .id')" \
  --closed true \
  --finished-at "2026-01-28"

# ============================================================================
# Gear - 2 grinders, 3 brewers, and 3 filter papers
# ============================================================================

# Grinders
./target/debug/brewlog gear add \
  --category "grinder" \
  --make "Comandante" \
  --model "C40 MK4"

./target/debug/brewlog gear add \
  --category "grinder" \
  --make "1Zpresso" \
  --model "J-Max"

# Brewers
./target/debug/brewlog gear add \
  --category "brewer" \
  --make "Hario" \
  --model "V60 02"

./target/debug/brewlog gear add \
  --category "brewer" \
  --make "AeroPress" \
  --model "Original"

./target/debug/brewlog gear add \
  --category "brewer" \
  --make "Fellow" \
  --model "Stagg XF"

# Filter Papers
./target/debug/brewlog gear add \
  --category "filter_paper" \
  --make "Hario" \
  --model "V60 Tabbed 02"

./target/debug/brewlog gear add \
  --category "filter_paper" \
  --make "Sibarist" \
  --model "FAST Specialty 02"

./target/debug/brewlog gear add \
  --category "filter_paper" \
  --make "Fellow" \
  --model "Stagg XF Filters"

# ============================================================================
# Brews - Sample brews using open bags with realistic ratios (1:15 to 1:17)
# ============================================================================

# Standard V60 brew - Ben Saïd Natural with Comandante (ratio 1:16.7)
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Ben Saïd Natural") | .id')" \
  --coffee-weight 15.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="C40 MK4") | .id')" \
  --grind-setting 24.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 02") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 Tabbed 02") | .id')" \
  --water-volume 250 \
  --water-temp 92.0

# AeroPress brew - Finca Tamana with J-Max (ratio 1:15)
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Finca Tamana Washed") | .id')" \
  --coffee-weight 17.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="J-Max") | .id')" \
  --grind-setting 20.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Original") | .id')" \
  --water-volume 255 \
  --water-temp 88.0

# Double V60 brew - La Linda with Comandante (ratio 1:16.7)
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="La Linda") | .id')" \
  --coffee-weight 30.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="C40 MK4") | .id')" \
  --grind-setting 25.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 02") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="FAST Specialty 02") | .id')" \
  --water-volume 500 \
  --water-temp 91.0

# Stagg XF brew - Guji Highland with J-Max (ratio 1:16.7)
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Guji Highland") | .id')" \
  --coffee-weight 18.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="J-Max") | .id')" \
  --grind-setting 22.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Stagg XF") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Stagg XF Filters") | .id')" \
  --water-volume 300 \
  --water-temp 93.0

# Light V60 brew - Red Brick Espresso with Comandante (ratio 1:16.7)
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Red Brick Espresso") | .id')" \
  --coffee-weight 12.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="C40 MK4") | .id')" \
  --grind-setting 26.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 02") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 Tabbed 02") | .id')" \
  --water-volume 200 \
  --water-temp 94.0

# AeroPress inverted - El Paraiso Anaerobic with J-Max (ratio 1:15)
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="El Paraiso 92 Anaerobic") | .id')" \
  --coffee-weight 15.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="J-Max") | .id')" \
  --grind-setting 18.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Original") | .id')" \
  --water-volume 225 \
  --water-temp 85.0

# V60 brew - Suke Quto with J-Max (ratio 1:16)
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Suke Quto") | .id')" \
  --coffee-weight 20.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="J-Max") | .id')" \
  --grind-setting 21.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 02") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="FAST Specialty 02") | .id')" \
  --water-volume 320 \
  --water-temp 92.0

# Stagg XF brew - Simbi with Comandante (ratio 1:16)
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Simbi") | .id')" \
  --coffee-weight 16.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="C40 MK4") | .id')" \
  --grind-setting 23.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Stagg XF") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Stagg XF Filters") | .id')" \
  --water-volume 256 \
  --water-temp 90.0

# ============================================================================
# Timestamp Distribution - Spread data over the last 6 months
# ============================================================================

DB_FILE="${DATABASE_URL:-brewlog.db}"
# Strip sqlite:// prefix if present
DB_FILE="${DB_FILE#sqlite://}"

echo "Distributing timestamps over the last 6 months..."

sqlite3 "$DB_FILE" <<'ENDSQL'
.timeout 5000
-- Today is approximately 2026-02-02
-- Spread data from 2025-08-01 to 2026-02-02 (6 months)

-- ============================================================================
-- GEAR: Added early (months 1-2) - you buy equipment before brewing
-- ============================================================================
UPDATE gear SET
  created_at = datetime('2025-08-05 10:00:00'),
  updated_at = datetime('2025-08-05 10:00:00')
WHERE model = 'C40 MK4';  -- Comandante grinder

UPDATE gear SET
  created_at = datetime('2025-08-10 14:30:00'),
  updated_at = datetime('2025-08-10 14:30:00')
WHERE model = 'V60 02';  -- Hario V60

UPDATE gear SET
  created_at = datetime('2025-08-15 09:00:00'),
  updated_at = datetime('2025-08-15 09:00:00')
WHERE model = 'Original';  -- AeroPress

UPDATE gear SET
  created_at = datetime('2025-09-20 11:00:00'),
  updated_at = datetime('2025-09-20 11:00:00')
WHERE model = 'J-Max';  -- 1Zpresso grinder (second grinder)

UPDATE gear SET
  created_at = datetime('2025-10-12 16:00:00'),
  updated_at = datetime('2025-10-12 16:00:00')
WHERE model = 'Stagg XF';  -- Fellow dripper (upgrade)

UPDATE gear SET
  created_at = datetime('2025-08-10 14:45:00'),
  updated_at = datetime('2025-08-10 14:45:00')
WHERE model = 'V60 Tabbed 02';  -- Hario filter papers (bought with V60)

UPDATE gear SET
  created_at = datetime('2025-10-01 09:00:00'),
  updated_at = datetime('2025-10-01 09:00:00')
WHERE model = 'FAST Specialty 02';  -- Sibarist upgrade papers

UPDATE gear SET
  created_at = datetime('2025-10-12 16:15:00'),
  updated_at = datetime('2025-10-12 16:15:00')
WHERE model = 'Stagg XF Filters';  -- Fellow filters (bought with Stagg XF)

-- ============================================================================
-- ROASTERS & ROASTS: Discovered over months 1-6
-- Order: Early favorites first, newer discoveries later
-- ============================================================================

-- Month 1 (Aug 2025): First roasters discovered
UPDATE roasters SET created_at = datetime('2025-08-03 08:00:00')
WHERE name = 'Tim Wendelboe';
UPDATE roasts SET created_at = datetime('2025-08-03 08:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Tim Wendelboe');

UPDATE roasters SET created_at = datetime('2025-08-08 12:00:00')
WHERE name = 'Square Mile Coffee';
UPDATE roasts SET created_at = datetime('2025-08-08 12:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Square Mile Coffee');

-- Month 2 (Sep 2025): More exploration
UPDATE roasters SET created_at = datetime('2025-09-05 10:00:00')
WHERE name = 'Coffee Collective';
UPDATE roasts SET created_at = datetime('2025-09-05 10:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Coffee Collective');

UPDATE roasters SET created_at = datetime('2025-09-15 14:00:00')
WHERE name = 'Assembly Coffee';
UPDATE roasts SET created_at = datetime('2025-09-15 14:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Assembly Coffee');

UPDATE roasters SET created_at = datetime('2025-09-25 09:00:00')
WHERE name = 'Bonanza Coffee';
UPDATE roasts SET created_at = datetime('2025-09-25 09:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Bonanza Coffee');

-- Month 3 (Oct 2025): Nordic deep dive
UPDATE roasters SET created_at = datetime('2025-10-05 11:00:00')
WHERE name = 'Drop Coffee';
UPDATE roasts SET created_at = datetime('2025-10-05 11:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Drop Coffee');

UPDATE roasters SET created_at = datetime('2025-10-15 13:00:00')
WHERE name = 'La Cabra';
UPDATE roasts SET created_at = datetime('2025-10-15 13:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'La Cabra');

UPDATE roasters SET created_at = datetime('2025-10-28 16:00:00')
WHERE name = 'April Coffee';
UPDATE roasts SET created_at = datetime('2025-10-28 16:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'April Coffee');

-- Month 4 (Nov 2025): European expansion
UPDATE roasters SET created_at = datetime('2025-11-02 10:00:00')
WHERE name = 'Dak Coffee Roasters';
UPDATE roasts SET created_at = datetime('2025-11-02 10:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Dak Coffee Roasters');

UPDATE roasters SET created_at = datetime('2025-11-12 15:00:00')
WHERE name = 'Friedhats';
UPDATE roasts SET created_at = datetime('2025-11-12 15:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Friedhats');

UPDATE roasters SET created_at = datetime('2025-11-22 09:00:00')
WHERE name = 'Origin Coffee';
UPDATE roasts SET created_at = datetime('2025-11-22 09:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Origin Coffee');

-- Month 5 (Dec 2025): Holiday discoveries
UPDATE roasters SET created_at = datetime('2025-12-05 11:00:00')
WHERE name = 'Dark Arts Coffee';
UPDATE roasts SET created_at = datetime('2025-12-05 11:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Dark Arts Coffee');

UPDATE roasters SET created_at = datetime('2025-12-15 14:00:00')
WHERE name = 'KAWA Coffee';
UPDATE roasts SET created_at = datetime('2025-12-15 14:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'KAWA Coffee');

UPDATE roasters SET created_at = datetime('2025-12-28 10:00:00')
WHERE name = 'Stow Coffee';
UPDATE roasts SET created_at = datetime('2025-12-28 10:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Stow Coffee');

-- Month 6 (Jan 2026): New year additions
UPDATE roasters SET created_at = datetime('2026-01-08 12:00:00')
WHERE name = 'Bows Coffee';
UPDATE roasts SET created_at = datetime('2026-01-08 12:05:00')
WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Bows Coffee');

-- ============================================================================
-- CAFES: Visited over months 1-6, spread across 5 cities
-- ============================================================================

-- Month 1 (Aug 2025): First London cafes
UPDATE cafes SET
  created_at = datetime('2025-08-12 11:00:00'),
  updated_at = datetime('2025-08-12 11:00:00')
WHERE name = 'Monmouth Coffee';

UPDATE cafes SET
  created_at = datetime('2025-08-20 14:30:00'),
  updated_at = datetime('2025-08-20 14:30:00')
WHERE name = 'Prufrock Coffee';

-- Month 2 (Sep 2025): Berlin trip
UPDATE cafes SET
  created_at = datetime('2025-09-08 10:00:00'),
  updated_at = datetime('2025-09-08 10:00:00')
WHERE name = 'The Barn';

UPDATE cafes SET
  created_at = datetime('2025-09-10 15:00:00'),
  updated_at = datetime('2025-09-10 15:00:00')
WHERE name = 'Companion Coffee';

-- Month 3 (Oct 2025): Munich visit
UPDATE cafes SET
  created_at = datetime('2025-10-07 09:30:00'),
  updated_at = datetime('2025-10-07 09:30:00')
WHERE name = 'Man Versus Machine';

UPDATE cafes SET
  created_at = datetime('2025-10-09 11:00:00'),
  updated_at = datetime('2025-10-09 11:00:00')
WHERE name = 'Vits der Kaffee';

-- Month 4 (Nov 2025): Madrid trip
UPDATE cafes SET
  created_at = datetime('2025-11-05 12:00:00'),
  updated_at = datetime('2025-11-05 12:00:00')
WHERE name = 'Hola Coffee';

UPDATE cafes SET
  created_at = datetime('2025-11-07 10:30:00'),
  updated_at = datetime('2025-11-07 10:30:00')
WHERE name = 'Toma Café';

-- Month 6 (Jan 2026): Bristol day trip
UPDATE cafes SET
  created_at = datetime('2026-01-11 10:00:00'),
  updated_at = datetime('2026-01-11 10:00:00')
WHERE name = 'Full Court Press';

UPDATE cafes SET
  created_at = datetime('2026-01-11 14:00:00'),
  updated_at = datetime('2026-01-11 14:00:00')
WHERE name = 'Small Street Espresso';

-- ============================================================================
-- CUPS: Spread to match cafe visit dates
-- Each cup is dated around the time the cafe was visited
-- ============================================================================

-- Monmouth Coffee (London, Aug 2025) - 2 cups
UPDATE cups SET
  created_at = datetime('2025-08-12 11:30:00'),
  updated_at = datetime('2025-08-12 11:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Monmouth Coffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Red Brick Espresso');

UPDATE cups SET
  created_at = datetime('2025-08-12 12:15:00'),
  updated_at = datetime('2025-08-12 12:15:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Monmouth Coffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Kamwangi');

-- Prufrock Coffee (London, Aug 2025) - 3 cups
UPDATE cups SET
  created_at = datetime('2025-08-20 14:45:00'),
  updated_at = datetime('2025-08-20 14:45:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Prufrock Coffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Kochere');

UPDATE cups SET
  created_at = datetime('2025-08-20 15:30:00'),
  updated_at = datetime('2025-08-20 15:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Prufrock Coffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'La Laja');

UPDATE cups SET
  created_at = datetime('2025-08-25 10:00:00'),
  updated_at = datetime('2025-08-25 10:00:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Prufrock Coffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Ben Saïd Natural');

-- The Barn (Berlin, Sep 2025) - 3 cups
UPDATE cups SET
  created_at = datetime('2025-09-08 10:30:00'),
  updated_at = datetime('2025-09-08 10:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'The Barn')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Gatomboya');

UPDATE cups SET
  created_at = datetime('2025-09-08 15:00:00'),
  updated_at = datetime('2025-09-08 15:00:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'The Barn')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Los Pirineos');

UPDATE cups SET
  created_at = datetime('2025-09-09 09:30:00'),
  updated_at = datetime('2025-09-09 09:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'The Barn')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Halo Beriti');

-- Companion Coffee (Berlin, Sep 2025) - 2 cups
UPDATE cups SET
  created_at = datetime('2025-09-10 15:30:00'),
  updated_at = datetime('2025-09-10 15:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Companion Coffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'El Paraiso 92 Anaerobic');

UPDATE cups SET
  created_at = datetime('2025-09-10 16:15:00'),
  updated_at = datetime('2025-09-10 16:15:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Companion Coffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Oreti SL28');

-- Man Versus Machine (Munich, Oct 2025) - 2 cups
UPDATE cups SET
  created_at = datetime('2025-10-07 10:00:00'),
  updated_at = datetime('2025-10-07 10:00:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Man Versus Machine')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Finca Tamana Washed');

UPDATE cups SET
  created_at = datetime('2025-10-07 14:30:00'),
  updated_at = datetime('2025-10-07 14:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Man Versus Machine')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'La Esmeralda Geisha');

-- Vits der Kaffee (Munich, Oct 2025) - 1 cup
UPDATE cups SET
  created_at = datetime('2025-10-09 11:30:00'),
  updated_at = datetime('2025-10-09 11:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Vits der Kaffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'San Fermin');

-- Hola Coffee (Madrid, Nov 2025) - 2 cups
UPDATE cups SET
  created_at = datetime('2025-11-05 12:30:00'),
  updated_at = datetime('2025-11-05 12:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Hola Coffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Cerro Azul');

UPDATE cups SET
  created_at = datetime('2025-11-06 10:00:00'),
  updated_at = datetime('2025-11-06 10:00:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Hola Coffee')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Guji Highland');

-- Toma Café (Madrid, Nov 2025) - 1 cup
UPDATE cups SET
  created_at = datetime('2025-11-07 11:00:00'),
  updated_at = datetime('2025-11-07 11:00:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Toma Café')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Daterra Sweet Collection');

-- Full Court Press (Bristol, Jan 2026) - 2 cups
UPDATE cups SET
  created_at = datetime('2026-01-11 10:30:00'),
  updated_at = datetime('2026-01-11 10:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Full Court Press')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Suke Quto');

UPDATE cups SET
  created_at = datetime('2026-01-11 11:15:00'),
  updated_at = datetime('2026-01-11 11:15:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Full Court Press')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Death to Decaf');

-- Small Street Espresso (Bristol, Jan 2026) - 2 cups
UPDATE cups SET
  created_at = datetime('2026-01-11 14:30:00'),
  updated_at = datetime('2026-01-11 14:30:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Small Street Espresso')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'Simbi');

UPDATE cups SET
  created_at = datetime('2026-01-11 15:00:00'),
  updated_at = datetime('2026-01-11 15:00:00')
WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Small Street Espresso')
  AND roast_id = (SELECT id FROM roasts WHERE name = 'La Chumeca');

-- ============================================================================
-- BAGS: Purchased after roasts exist, spread over months 2-6
-- Roast dates should be slightly before created_at (bag bought after roasting)
-- ============================================================================

-- Gatomboya (Bonanza) - Sep 2025, finished in Oct
UPDATE bags SET
  created_at = datetime('2025-09-28 10:00:00'),
  updated_at = datetime('2025-10-15 09:00:00'),
  roast_date = '2025-09-20',
  finished_at = '2025-10-15'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Gatomboya');

-- Kochere (Assembly) - early Oct, finished mid-Oct
UPDATE bags SET
  created_at = datetime('2025-10-02 11:00:00'),
  updated_at = datetime('2025-10-20 14:00:00'),
  roast_date = '2025-09-25',
  finished_at = '2025-10-20'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Kochere');

-- Daterra Sweet Collection (Coffee Collective) - Oct, finished Nov
UPDATE bags SET
  created_at = datetime('2025-10-10 14:00:00'),
  updated_at = datetime('2025-11-05 10:00:00'),
  roast_date = '2025-10-05',
  finished_at = '2025-11-05'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Daterra Sweet Collection');

-- Halo Beriti (La Cabra) - late Oct, finished Nov
UPDATE bags SET
  created_at = datetime('2025-10-25 09:00:00'),
  updated_at = datetime('2025-11-12 16:00:00'),
  roast_date = '2025-10-18',
  finished_at = '2025-11-12'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Halo Beriti');

-- Ben Saïd Natural (Tim Wendelboe) - Nov, still open
UPDATE bags SET
  created_at = datetime('2025-11-15 10:00:00'),
  updated_at = datetime('2025-11-15 10:00:00'),
  roast_date = '2025-11-10'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Ben Saïd Natural');

-- Finca Tamana Washed (Tim Wendelboe) - Nov, still open
UPDATE bags SET
  created_at = datetime('2025-11-20 13:00:00'),
  updated_at = datetime('2025-11-20 13:00:00'),
  roast_date = '2025-11-15'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Finca Tamana Washed');

-- La Linda (Drop Coffee) - Dec, still open
UPDATE bags SET
  created_at = datetime('2025-12-10 11:00:00'),
  updated_at = datetime('2025-12-10 11:00:00'),
  roast_date = '2025-12-05'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'La Linda');

-- Guji Highland (April Coffee) - Dec, still open
UPDATE bags SET
  created_at = datetime('2025-12-18 14:00:00'),
  updated_at = datetime('2025-12-18 14:00:00'),
  roast_date = '2025-12-12'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Guji Highland');

-- Red Brick Espresso (Square Mile) - late Dec, still open
UPDATE bags SET
  created_at = datetime('2025-12-28 09:00:00'),
  updated_at = datetime('2025-12-28 09:00:00'),
  roast_date = '2025-12-22'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Red Brick Espresso');

-- El Paraiso 92 Anaerobic (Dak) - Jan 2026, still open
UPDATE bags SET
  created_at = datetime('2026-01-10 10:00:00'),
  updated_at = datetime('2026-01-10 10:00:00'),
  roast_date = '2026-01-05'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'El Paraiso 92 Anaerobic');

-- Suke Quto (Stow) - Jan 2026, still open
UPDATE bags SET
  created_at = datetime('2026-01-15 12:00:00'),
  updated_at = datetime('2026-01-15 12:00:00'),
  roast_date = '2026-01-10'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Suke Quto');

-- Simbi (Bows Coffee) - late Jan 2026, still open
UPDATE bags SET
  created_at = datetime('2026-01-22 15:00:00'),
  updated_at = datetime('2026-01-22 15:00:00'),
  roast_date = '2026-01-18'
WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Simbi');

-- ============================================================================
-- BREWS: Created after bags exist, spread over recent months
-- ============================================================================

-- Brew 1: Ben Saïd Natural V60 - mid Nov
UPDATE brews SET
  created_at = datetime('2025-11-18 08:30:00'),
  updated_at = datetime('2025-11-18 08:30:00')
WHERE bag_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Ben Saïd Natural'))
  AND water_volume = 250;

-- Brew 2: Finca Tamana AeroPress - late Nov
UPDATE brews SET
  created_at = datetime('2025-11-25 09:00:00'),
  updated_at = datetime('2025-11-25 09:00:00')
WHERE bag_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Finca Tamana Washed'))
  AND water_volume = 255;

-- Brew 3: La Linda double V60 - mid Dec
UPDATE brews SET
  created_at = datetime('2025-12-15 10:00:00'),
  updated_at = datetime('2025-12-15 10:00:00')
WHERE bag_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'La Linda'))
  AND water_volume = 500;

-- Brew 4: Guji Highland Stagg XF - late Dec
UPDATE brews SET
  created_at = datetime('2025-12-22 08:00:00'),
  updated_at = datetime('2025-12-22 08:00:00')
WHERE bag_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Guji Highland'))
  AND water_volume = 300;

-- Brew 5: Red Brick V60 - early Jan
UPDATE brews SET
  created_at = datetime('2026-01-02 09:30:00'),
  updated_at = datetime('2026-01-02 09:30:00')
WHERE bag_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Red Brick Espresso'))
  AND water_volume = 200;

-- Brew 6: El Paraiso AeroPress - mid Jan
UPDATE brews SET
  created_at = datetime('2026-01-12 08:15:00'),
  updated_at = datetime('2026-01-12 08:15:00')
WHERE bag_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'El Paraiso 92 Anaerobic'))
  AND water_volume = 225;

-- Brew 7: Suke Quto V60 - late Jan
UPDATE brews SET
  created_at = datetime('2026-01-20 09:00:00'),
  updated_at = datetime('2026-01-20 09:00:00')
WHERE bag_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Suke Quto'))
  AND water_volume = 320;

-- Brew 8: Simbi Stagg XF - recent (late Jan)
UPDATE brews SET
  created_at = datetime('2026-01-28 08:45:00'),
  updated_at = datetime('2026-01-28 08:45:00')
WHERE bag_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Simbi'))
  AND water_volume = 256;

-- ============================================================================
-- TIMELINE_EVENTS: Update to match entity creation times
-- ============================================================================

-- Gear timeline events
UPDATE timeline_events SET occurred_at = datetime('2025-08-05 10:00:00')
WHERE entity_type = 'gear' AND entity_id = (SELECT id FROM gear WHERE model = 'C40 MK4');

UPDATE timeline_events SET occurred_at = datetime('2025-08-10 14:30:00')
WHERE entity_type = 'gear' AND entity_id = (SELECT id FROM gear WHERE model = 'V60 02');

UPDATE timeline_events SET occurred_at = datetime('2025-08-15 09:00:00')
WHERE entity_type = 'gear' AND entity_id = (SELECT id FROM gear WHERE model = 'Original');

UPDATE timeline_events SET occurred_at = datetime('2025-09-20 11:00:00')
WHERE entity_type = 'gear' AND entity_id = (SELECT id FROM gear WHERE model = 'J-Max');

UPDATE timeline_events SET occurred_at = datetime('2025-10-12 16:00:00')
WHERE entity_type = 'gear' AND entity_id = (SELECT id FROM gear WHERE model = 'Stagg XF');

UPDATE timeline_events SET occurred_at = datetime('2025-08-10 14:45:00')
WHERE entity_type = 'gear' AND entity_id = (SELECT id FROM gear WHERE model = 'V60 Tabbed 02');

UPDATE timeline_events SET occurred_at = datetime('2025-10-01 09:00:00')
WHERE entity_type = 'gear' AND entity_id = (SELECT id FROM gear WHERE model = 'FAST Specialty 02');

UPDATE timeline_events SET occurred_at = datetime('2025-10-12 16:15:00')
WHERE entity_type = 'gear' AND entity_id = (SELECT id FROM gear WHERE model = 'Stagg XF Filters');

-- Roaster timeline events
UPDATE timeline_events SET occurred_at = datetime('2025-08-03 08:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Tim Wendelboe');

UPDATE timeline_events SET occurred_at = datetime('2025-08-08 12:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Square Mile Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-09-05 10:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Coffee Collective');

UPDATE timeline_events SET occurred_at = datetime('2025-09-15 14:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Assembly Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-09-25 09:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Bonanza Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-10-05 11:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Drop Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-10-15 13:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'La Cabra');

UPDATE timeline_events SET occurred_at = datetime('2025-10-28 16:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'April Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-11-02 10:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Dak Coffee Roasters');

UPDATE timeline_events SET occurred_at = datetime('2025-11-12 15:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Friedhats');

UPDATE timeline_events SET occurred_at = datetime('2025-11-22 09:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Origin Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-12-05 11:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Dark Arts Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-12-15 14:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'KAWA Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-12-28 10:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Stow Coffee');

UPDATE timeline_events SET occurred_at = datetime('2026-01-08 12:00:00')
WHERE entity_type = 'roaster' AND entity_id = (SELECT id FROM roasters WHERE name = 'Bows Coffee');

-- Roast timeline events (match roaster times + 5 min)
UPDATE timeline_events SET occurred_at = datetime('2025-08-03 08:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Tim Wendelboe'));

UPDATE timeline_events SET occurred_at = datetime('2025-08-08 12:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Square Mile Coffee'));

UPDATE timeline_events SET occurred_at = datetime('2025-09-05 10:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Coffee Collective'));

UPDATE timeline_events SET occurred_at = datetime('2025-09-15 14:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Assembly Coffee'));

UPDATE timeline_events SET occurred_at = datetime('2025-09-25 09:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Bonanza Coffee'));

UPDATE timeline_events SET occurred_at = datetime('2025-10-05 11:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Drop Coffee'));

UPDATE timeline_events SET occurred_at = datetime('2025-10-15 13:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'La Cabra'));

UPDATE timeline_events SET occurred_at = datetime('2025-10-28 16:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'April Coffee'));

UPDATE timeline_events SET occurred_at = datetime('2025-11-02 10:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Dak Coffee Roasters'));

UPDATE timeline_events SET occurred_at = datetime('2025-11-12 15:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Friedhats'));

UPDATE timeline_events SET occurred_at = datetime('2025-11-22 09:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Origin Coffee'));

UPDATE timeline_events SET occurred_at = datetime('2025-12-05 11:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Dark Arts Coffee'));

UPDATE timeline_events SET occurred_at = datetime('2025-12-15 14:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'KAWA Coffee'));

UPDATE timeline_events SET occurred_at = datetime('2025-12-28 10:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Stow Coffee'));

UPDATE timeline_events SET occurred_at = datetime('2026-01-08 12:05:00')
WHERE entity_type = 'roast' AND entity_id IN (SELECT id FROM roasts WHERE roaster_id = (SELECT id FROM roasters WHERE name = 'Bows Coffee'));

-- Cafe timeline events (match cafe created_at)
UPDATE timeline_events SET occurred_at = datetime('2025-08-12 11:00:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'Monmouth Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-08-20 14:30:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'Prufrock Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-09-08 10:00:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'The Barn');

UPDATE timeline_events SET occurred_at = datetime('2025-09-10 15:00:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'Companion Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-10-07 09:30:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'Man Versus Machine');

UPDATE timeline_events SET occurred_at = datetime('2025-10-09 11:00:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'Vits der Kaffee');

UPDATE timeline_events SET occurred_at = datetime('2025-11-05 12:00:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'Hola Coffee');

UPDATE timeline_events SET occurred_at = datetime('2025-11-07 10:30:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'Toma Café');

UPDATE timeline_events SET occurred_at = datetime('2026-01-11 10:00:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'Full Court Press');

UPDATE timeline_events SET occurred_at = datetime('2026-01-11 14:00:00')
WHERE entity_type = 'cafe' AND entity_id = (SELECT id FROM cafes WHERE name = 'Small Street Espresso');

-- Rebuild cafe timeline details_json to include Position field
UPDATE timeline_events SET details_json = (
  SELECT '[{"label":"City","value":"' || c.city || '"},{"label":"Country","value":"' || c.country || '"},{"label":"Website","value":"' || COALESCE(NULLIF(c.website, ''), '—') || '"},{"label":"Position","value":"https://www.google.com/maps?q=' || c.latitude || ',' || c.longitude || '"}]'
  FROM cafes c WHERE c.id = timeline_events.entity_id
)
WHERE entity_type = 'cafe';

-- Bag timeline events (match bag created_at)
UPDATE timeline_events SET occurred_at = datetime('2025-09-28 10:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Gatomboya'));

UPDATE timeline_events SET occurred_at = datetime('2025-10-02 11:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Kochere'));

UPDATE timeline_events SET occurred_at = datetime('2025-10-10 14:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Daterra Sweet Collection'));

UPDATE timeline_events SET occurred_at = datetime('2025-10-25 09:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Halo Beriti'));

UPDATE timeline_events SET occurred_at = datetime('2025-11-15 10:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Ben Saïd Natural'));

UPDATE timeline_events SET occurred_at = datetime('2025-11-20 13:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Finca Tamana Washed'));

UPDATE timeline_events SET occurred_at = datetime('2025-12-10 11:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'La Linda'));

UPDATE timeline_events SET occurred_at = datetime('2025-12-18 14:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Guji Highland'));

UPDATE timeline_events SET occurred_at = datetime('2025-12-28 09:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Red Brick Espresso'));

UPDATE timeline_events SET occurred_at = datetime('2026-01-10 10:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'El Paraiso 92 Anaerobic'));

UPDATE timeline_events SET occurred_at = datetime('2026-01-15 12:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Suke Quto'));

UPDATE timeline_events SET occurred_at = datetime('2026-01-22 15:00:00')
WHERE entity_type = 'bag' AND entity_id = (SELECT id FROM bags WHERE roast_id = (SELECT id FROM roasts WHERE name = 'Simbi'));

-- Brew timeline events (match brew created_at)
UPDATE timeline_events SET occurred_at = datetime('2025-11-18 08:30:00')
WHERE entity_type = 'brew' AND entity_id = (
  SELECT b.id FROM brews b
  JOIN bags ba ON b.bag_id = ba.id
  JOIN roasts r ON ba.roast_id = r.id
  WHERE r.name = 'Ben Saïd Natural' AND b.water_volume = 250
);

UPDATE timeline_events SET occurred_at = datetime('2025-11-25 09:00:00')
WHERE entity_type = 'brew' AND entity_id = (
  SELECT b.id FROM brews b
  JOIN bags ba ON b.bag_id = ba.id
  JOIN roasts r ON ba.roast_id = r.id
  WHERE r.name = 'Finca Tamana Washed' AND b.water_volume = 255
);

UPDATE timeline_events SET occurred_at = datetime('2025-12-15 10:00:00')
WHERE entity_type = 'brew' AND entity_id = (
  SELECT b.id FROM brews b
  JOIN bags ba ON b.bag_id = ba.id
  JOIN roasts r ON ba.roast_id = r.id
  WHERE r.name = 'La Linda' AND b.water_volume = 500
);

UPDATE timeline_events SET occurred_at = datetime('2025-12-22 08:00:00')
WHERE entity_type = 'brew' AND entity_id = (
  SELECT b.id FROM brews b
  JOIN bags ba ON b.bag_id = ba.id
  JOIN roasts r ON ba.roast_id = r.id
  WHERE r.name = 'Guji Highland' AND b.water_volume = 300
);

UPDATE timeline_events SET occurred_at = datetime('2026-01-02 09:30:00')
WHERE entity_type = 'brew' AND entity_id = (
  SELECT b.id FROM brews b
  JOIN bags ba ON b.bag_id = ba.id
  JOIN roasts r ON ba.roast_id = r.id
  WHERE r.name = 'Red Brick Espresso' AND b.water_volume = 200
);

UPDATE timeline_events SET occurred_at = datetime('2026-01-12 08:15:00')
WHERE entity_type = 'brew' AND entity_id = (
  SELECT b.id FROM brews b
  JOIN bags ba ON b.bag_id = ba.id
  JOIN roasts r ON ba.roast_id = r.id
  WHERE r.name = 'El Paraiso 92 Anaerobic' AND b.water_volume = 225
);

UPDATE timeline_events SET occurred_at = datetime('2026-01-20 09:00:00')
WHERE entity_type = 'brew' AND entity_id = (
  SELECT b.id FROM brews b
  JOIN bags ba ON b.bag_id = ba.id
  JOIN roasts r ON ba.roast_id = r.id
  WHERE r.name = 'Suke Quto' AND b.water_volume = 320
);

UPDATE timeline_events SET occurred_at = datetime('2026-01-28 08:45:00')
WHERE entity_type = 'brew' AND entity_id = (
  SELECT b.id FROM brews b
  JOIN bags ba ON b.bag_id = ba.id
  JOIN roasts r ON ba.roast_id = r.id
  WHERE r.name = 'Simbi' AND b.water_volume = 256
);

-- Cup timeline events (match cup created_at)
-- Monmouth Coffee cups
UPDATE timeline_events SET occurred_at = datetime('2025-08-12 11:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Monmouth Coffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Red Brick Espresso')
);

UPDATE timeline_events SET occurred_at = datetime('2025-08-12 12:15:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Monmouth Coffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Kamwangi')
);

-- Prufrock Coffee cups
UPDATE timeline_events SET occurred_at = datetime('2025-08-20 14:45:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Prufrock Coffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Kochere')
);

UPDATE timeline_events SET occurred_at = datetime('2025-08-20 15:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Prufrock Coffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'La Laja')
);

UPDATE timeline_events SET occurred_at = datetime('2025-08-25 10:00:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Prufrock Coffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Ben Saïd Natural')
);

-- The Barn cups
UPDATE timeline_events SET occurred_at = datetime('2025-09-08 10:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'The Barn')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Gatomboya')
);

UPDATE timeline_events SET occurred_at = datetime('2025-09-08 15:00:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'The Barn')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Los Pirineos')
);

UPDATE timeline_events SET occurred_at = datetime('2025-09-09 09:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'The Barn')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Halo Beriti')
);

-- Companion Coffee cups
UPDATE timeline_events SET occurred_at = datetime('2025-09-10 15:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Companion Coffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'El Paraiso 92 Anaerobic')
);

UPDATE timeline_events SET occurred_at = datetime('2025-09-10 16:15:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Companion Coffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Oreti SL28')
);

-- Man Versus Machine cups
UPDATE timeline_events SET occurred_at = datetime('2025-10-07 10:00:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Man Versus Machine')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Finca Tamana Washed')
);

UPDATE timeline_events SET occurred_at = datetime('2025-10-07 14:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Man Versus Machine')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'La Esmeralda Geisha')
);

-- Vits der Kaffee cup
UPDATE timeline_events SET occurred_at = datetime('2025-10-09 11:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Vits der Kaffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'San Fermin')
);

-- Hola Coffee cups
UPDATE timeline_events SET occurred_at = datetime('2025-11-05 12:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Hola Coffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Cerro Azul')
);

UPDATE timeline_events SET occurred_at = datetime('2025-11-06 10:00:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Hola Coffee')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Guji Highland')
);

-- Toma Café cup
UPDATE timeline_events SET occurred_at = datetime('2025-11-07 11:00:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Toma Café')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Daterra Sweet Collection')
);

-- Full Court Press cups
UPDATE timeline_events SET occurred_at = datetime('2026-01-11 10:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Full Court Press')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Suke Quto')
);

UPDATE timeline_events SET occurred_at = datetime('2026-01-11 11:15:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Full Court Press')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Death to Decaf')
);

-- Small Street Espresso cups
UPDATE timeline_events SET occurred_at = datetime('2026-01-11 14:30:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Small Street Espresso')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'Simbi')
);

UPDATE timeline_events SET occurred_at = datetime('2026-01-11 15:00:00')
WHERE entity_type = 'cup' AND entity_id = (
  SELECT id FROM cups WHERE cafe_id = (SELECT id FROM cafes WHERE name = 'Small Street Espresso')
    AND roast_id = (SELECT id FROM roasts WHERE name = 'La Chumeca')
);

ENDSQL

echo
echo "Bootstrapped database with distributed timestamps"
echo
echo "Set token $BREWLOG_TOKEN to use the data added here."
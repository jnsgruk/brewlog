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

# ============================================================================
# Roasters & Roasts - 15 roasters with 2 roasts each, spread over 6 months
# ============================================================================

# Month 1 (Aug 2025): First roasters discovered

# Tim Wendelboe (Norway)
./target/debug/brewlog roaster add \
  --name "Tim Wendelboe" \
  --country "Norway" \
  --city "Oslo" \
  --homepage "https://timwendelboe.no" \
  --created-at "2025-08-03T08:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Tim Wendelboe") | .id')" \
  --name "Ben Saïd Natural" \
  --origin "Ethiopia" \
  --region "Sidamo" \
  --producer "Ben Saïd" \
  --process "Natural" \
  --tasting-notes "Bergamot, Apricot, Floral" \
  --created-at "2025-08-03T08:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Tim Wendelboe") | .id')" \
  --name "Finca Tamana Washed" \
  --origin "Colombia" \
  --region "El Pital, Huila" \
  --producer "Elias Roa" \
  --process "Washed" \
  --tasting-notes "Red Apple, Vanilla, Caramel" \
  --created-at "2025-08-03T08:05:00Z"

# Square Mile (UK)
./target/debug/brewlog roaster add \
  --name "Square Mile Coffee" \
  --country "United Kingdom" \
  --city "London" \
  --homepage "https://squaremilecoffee.com" \
  --created-at "2025-08-08T12:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Square Mile Coffee") | .id')" \
  --name "Red Brick Espresso" \
  --origin "Blend" \
  --region "Multiple Origins" \
  --producer "Various" \
  --process "Washed, Natural" \
  --tasting-notes "Berry, Chocolate, Citrus" \
  --created-at "2025-08-08T12:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Square Mile Coffee") | .id')" \
  --name "Kamwangi" \
  --origin "Kenya" \
  --region "Kirinyaga" \
  --producer "Kamwangi Factory" \
  --process "Washed" \
  --tasting-notes "Blackcurrant, Rhubarb, Blood Orange" \
  --created-at "2025-08-08T12:05:00Z"

# Month 2 (Sep 2025): More exploration

# Coffee Collective (Denmark)
./target/debug/brewlog roaster add \
  --name "Coffee Collective" \
  --country "Denmark" \
  --city "Copenhagen" \
  --homepage "https://coffeecollective.dk" \
  --created-at "2025-09-05T10:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Coffee Collective") | .id')" \
  --name "Daterra Sweet Collection" \
  --origin "Brazil" \
  --region "Cerrado" \
  --producer "Daterra" \
  --process "Pulped Natural" \
  --tasting-notes "Hazelnut, Milk Chocolate, Yellow Fruit" \
  --created-at "2025-09-05T10:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Coffee Collective") | .id')" \
  --name "Kieni" \
  --origin "Kenya" \
  --region "Nyeri" \
  --producer "Kieni Factory" \
  --process "Washed" \
  --tasting-notes "Currant, Black Tea, Grape" \
  --created-at "2025-09-05T10:05:00Z"

# Assembly Coffee (UK)
./target/debug/brewlog roaster add \
  --name "Assembly Coffee" \
  --country "United Kingdom" \
  --city "London" \
  --homepage "https://assemblycoffee.co.uk" \
  --created-at "2025-09-15T14:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Assembly Coffee") | .id')" \
  --name "Kochere" \
  --origin "Ethiopia" \
  --region "Yirgacheffe" \
  --producer "Kochere Region Growers" \
  --process "Washed" \
  --tasting-notes "Peach, Lemon, Jasmine" \
  --created-at "2025-09-15T14:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Assembly Coffee") | .id')" \
  --name "La Laja" \
  --origin "Mexico" \
  --region "Veracruz" \
  --producer "La Laja Estate" \
  --process "Natural" \
  --tasting-notes "Cherry, Milk Chocolate, Praline" \
  --created-at "2025-09-15T14:05:00Z"

# Bonanza Coffee (Germany)
./target/debug/brewlog roaster add \
  --name "Bonanza Coffee" \
  --country "Germany" \
  --city "Berlin" \
  --homepage "https://www.bonanzacoffee.de" \
  --created-at "2025-09-25T09:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Bonanza Coffee") | .id')" \
  --name "Gatomboya" \
  --origin "Kenya" \
  --region "Nyeri" \
  --producer "Gatomboya Cooperative" \
  --process "Washed" \
  --tasting-notes "Blackcurrant, Lime, Tomato" \
  --created-at "2025-09-25T09:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Bonanza Coffee") | .id')" \
  --name "Los Pirineos" \
  --origin "El Salvador" \
  --region "Usulután" \
  --producer "Gilberto Baraona" \
  --process "Honey" \
  --tasting-notes "Maple, Fudge, Green Apple" \
  --created-at "2025-09-25T09:05:00Z"

# Month 3 (Oct 2025): Nordic deep dive

# Drop Coffee (Sweden)
./target/debug/brewlog roaster add \
  --name "Drop Coffee" \
  --country "Sweden" \
  --city "Stockholm" \
  --homepage "https://dropcoffee.com" \
  --created-at "2025-10-05T11:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Drop Coffee") | .id')" \
  --name "La Linda" \
  --origin "Bolivia" \
  --region "Caranavi" \
  --producer "Pedro Rodriguez" \
  --process "Washed" \
  --tasting-notes "Red Apple, Caramel, Floral" \
  --created-at "2025-10-05T11:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Drop Coffee") | .id')" \
  --name "El Sunzita" \
  --origin "El Salvador" \
  --region "Ahuachapan" \
  --producer "Jorge Raul Rivera" \
  --process "Natural" \
  --tasting-notes "Strawberry, Mango, Dark Chocolate" \
  --created-at "2025-10-05T11:05:00Z"

# La Cabra (Denmark)
./target/debug/brewlog roaster add \
  --name "La Cabra" \
  --country "Denmark" \
  --city "Aarhus" \
  --homepage "https://www.lacabra.dk" \
  --created-at "2025-10-15T13:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="La Cabra") | .id')" \
  --name "Halo Beriti" \
  --origin "Ethiopia" \
  --region "Yirgacheffe" \
  --producer "Halo Beriti Cooperative" \
  --process "Washed" \
  --tasting-notes "Jasmine, Lemon, Stone Fruit" \
  --created-at "2025-10-15T13:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="La Cabra") | .id')" \
  --name "Cerro Azul" \
  --origin "Colombia" \
  --region "Valle del Cauca" \
  --producer "Granja La Esperanza" \
  --process "Washed" \
  --tasting-notes "Blueberry, Plum, Grapefruit" \
  --created-at "2025-10-15T13:05:00Z"

# April Coffee (Denmark)
./target/debug/brewlog roaster add \
  --name "April Coffee" \
  --country "Denmark" \
  --city "Copenhagen" \
  --homepage "https://aprilcoffeeroasters.com" \
  --created-at "2025-10-28T16:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="April Coffee") | .id')" \
  --name "El Salvador Pacamara" \
  --origin "El Salvador" \
  --region "Santa Ana" \
  --producer "Ernesto Menendez" \
  --process "Honey" \
  --tasting-notes "Grapefruit, Sugar Cane, Plum" \
  --created-at "2025-10-28T16:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="April Coffee") | .id')" \
  --name "Guji Highland" \
  --origin "Ethiopia" \
  --region "Guji" \
  --producer "Andualem Abebe" \
  --process "Natural" \
  --tasting-notes "Peach, Strawberry, Cream" \
  --created-at "2025-10-28T16:05:00Z"

# Month 4 (Nov 2025): European expansion

# Dak Coffee Roasters (Netherlands)
./target/debug/brewlog roaster add \
  --name "Dak Coffee Roasters" \
  --country "Netherlands" \
  --city "Amsterdam" \
  --homepage "https://www.dakcoffeeroasters.com" \
  --created-at "2025-11-02T10:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Dak Coffee Roasters") | .id')" \
  --name "El Paraiso 92 Anaerobic" \
  --origin "Colombia" \
  --region "Cauca" \
  --producer "Diego Bermudez" \
  --process "Thermal Shock Anaerobic" \
  --tasting-notes "Passionfruit, Raspberry, Yogurt" \
  --created-at "2025-11-02T10:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Dak Coffee Roasters") | .id')" \
  --name "Oreti SL28" \
  --origin "Kenya" \
  --region "Kirinyaga" \
  --producer "Oreti Estate" \
  --process "Washed" \
  --tasting-notes "Grapefruit, Blackcurrant, Plum" \
  --created-at "2025-11-02T10:05:00Z"

# Friedhats (Netherlands)
./target/debug/brewlog roaster add \
  --name "Friedhats" \
  --country "Netherlands" \
  --city "Amsterdam" \
  --homepage "https://friedhats.com" \
  --created-at "2025-11-12T15:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Friedhats") | .id')" \
  --name "Sidamo Guji" \
  --origin "Ethiopia" \
  --region "Guji" \
  --producer "Smallholders" \
  --process "Natural" \
  --tasting-notes "Peach, Raspberry, Rosehip" \
  --created-at "2025-11-12T15:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Friedhats") | .id')" \
  --name "La Esmeralda Geisha" \
  --origin "Panama" \
  --region "Boquete" \
  --producer "Hacienda La Esmeralda" \
  --process "Washed" \
  --tasting-notes "Jasmine, Bergamot, Papaya" \
  --created-at "2025-11-12T15:05:00Z"

# Origin Coffee (UK)
./target/debug/brewlog roaster add \
  --name "Origin Coffee" \
  --country "United Kingdom" \
  --city "Porthleven" \
  --homepage "https://origincoffee.co.uk" \
  --created-at "2025-11-22T09:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Origin Coffee") | .id')" \
  --name "San Fermin" \
  --origin "Colombia" \
  --region "Tolima" \
  --producer "San Fermin Smallholders" \
  --process "Washed" \
  --tasting-notes "Red Grape, Caramel, Blood Orange" \
  --created-at "2025-11-22T09:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Origin Coffee") | .id')" \
  --name "Aricha" \
  --origin "Ethiopia" \
  --region "Yirgacheffe" \
  --producer "Aricha Washing Station" \
  --process "Washed" \
  --tasting-notes "Honey, Peach, Black Tea" \
  --created-at "2025-11-22T09:05:00Z"

# Month 5 (Dec 2025): Holiday discoveries

# Dark Arts Coffee (UK)
./target/debug/brewlog roaster add \
  --name "Dark Arts Coffee" \
  --country "United Kingdom" \
  --city "London" \
  --homepage "https://www.darkartscoffee.co.uk" \
  --created-at "2025-12-05T11:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Dark Arts Coffee") | .id')" \
  --name "Death to Decaf" \
  --origin "Brazil" \
  --region "Minas Gerais" \
  --producer "Carmo de Minas" \
  --process "Swiss Water Decaf" \
  --tasting-notes "Cocoa, Cherry, Almond" \
  --created-at "2025-12-05T11:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Dark Arts Coffee") | .id')" \
  --name "Snoop" \
  --origin "Guatemala" \
  --region "Huehuetenango" \
  --producer "Various Smallholders" \
  --process "Washed" \
  --tasting-notes "Toffee, Green Apple, Plum" \
  --created-at "2025-12-05T11:05:00Z"

# KAWA Coffee (France)
./target/debug/brewlog roaster add \
  --name "KAWA Coffee" \
  --country "France" \
  --city "Paris" \
  --homepage "https://www.kawa.coffee" \
  --created-at "2025-12-15T14:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="KAWA Coffee") | .id')" \
  --name "Sudan Rume" \
  --origin "Colombia" \
  --region "Cauca" \
  --producer "Granja La Esperanza" \
  --process "Natural" \
  --tasting-notes "Strawberry, Cinnamon, Grape" \
  --created-at "2025-12-15T14:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="KAWA Coffee") | .id')" \
  --name "Arbegona" \
  --origin "Ethiopia" \
  --region "Sidama" \
  --producer "Arbegona Washing Station" \
  --process "Washed" \
  --tasting-notes "Violet, Apricot, Lemon" \
  --created-at "2025-12-15T14:05:00Z"

# Stow Coffee (Slovenia)
./target/debug/brewlog roaster add \
  --name "Stow Coffee" \
  --country "Slovenia" \
  --city "Ljubljana" \
  --homepage "https://www.stowcoffee.com" \
  --created-at "2025-12-28T10:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Stow Coffee") | .id')" \
  --name "Santa Barbara" \
  --origin "Honduras" \
  --region "Santa Barbara" \
  --producer "Benjamin Paz" \
  --process "Honey" \
  --tasting-notes "Red Currant, Honeydew, Cocoa" \
  --created-at "2025-12-28T10:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Stow Coffee") | .id')" \
  --name "Suke Quto" \
  --origin "Ethiopia" \
  --region "Guji" \
  --producer "Tesfaye Bekele" \
  --process "Natural" \
  --tasting-notes "Blackberry, Vanilla, Jasmine" \
  --created-at "2025-12-28T10:05:00Z"

# Month 6 (Jan 2026): New year additions

# Bows Coffee (Canada)
./target/debug/brewlog roaster add \
  --name "Bows Coffee" \
  --country "Canada" \
  --city "Victoria" \
  --homepage "https://bowscoffee.com" \
  --created-at "2026-01-08T12:00:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Bows Coffee") | .id')" \
  --name "La Chumeca" \
  --origin "Costa Rica" \
  --region "Tarrazú" \
  --producer "Doña Olga Jiménez" \
  --process "White Honey" \
  --tasting-notes "Mandarin, Honeycomb, Almond" \
  --created-at "2026-01-08T12:05:00Z"

./target/debug/brewlog roast add \
  --roaster-id "$(./target/debug/brewlog roaster list | jq -r '.[] | select(.name=="Bows Coffee") | .id')" \
  --name "Simbi" \
  --origin "Rwanda" \
  --region "Huye" \
  --producer "Simbi Co-op" \
  --process "Washed" \
  --tasting-notes "Black Tea, Orange, Cane Sugar" \
  --created-at "2026-01-08T12:05:00Z"

# ============================================================================
# Cafes - 10 cafes across London, Madrid, Berlin, Munich, and Bristol
# ============================================================================

# London - Monmouth Coffee (Borough Market) — Aug 2025
./target/debug/brewlog cafe add \
  --name "Monmouth Coffee" \
  --city "London" \
  --country "United Kingdom" \
  --latitude 51.5055 \
  --longitude -0.0910 \
  --website "https://www.monmouthcoffee.co.uk" \
  --created-at "2025-08-12T11:00:00Z"

# London - Prufrock Coffee — Aug 2025
./target/debug/brewlog cafe add \
  --name "Prufrock Coffee" \
  --city "London" \
  --country "United Kingdom" \
  --latitude 51.5246 \
  --longitude -0.1098 \
  --website "https://www.prufrockcoffee.com" \
  --created-at "2025-08-20T14:30:00Z"

# Berlin - The Barn — Sep 2025
./target/debug/brewlog cafe add \
  --name "The Barn" \
  --city "Berlin" \
  --country "Germany" \
  --latitude 52.5298 \
  --longitude 13.4020 \
  --website "https://thebarn.de" \
  --created-at "2025-09-08T10:00:00Z"

# Berlin - Companion Coffee — Sep 2025
./target/debug/brewlog cafe add \
  --name "Companion Coffee" \
  --city "Berlin" \
  --country "Germany" \
  --latitude 52.4952 \
  --longitude 13.4188 \
  --website "https://www.companion.coffee" \
  --created-at "2025-09-10T15:00:00Z"

# Munich - Man Versus Machine — Oct 2025
./target/debug/brewlog cafe add \
  --name "Man Versus Machine" \
  --city "Munich" \
  --country "Germany" \
  --latitude 48.1310 \
  --longitude 11.5690 \
  --website "https://www.mvsmcoffee.de" \
  --created-at "2025-10-07T09:30:00Z"

# Munich - Vits der Kaffee — Oct 2025
./target/debug/brewlog cafe add \
  --name "Vits der Kaffee" \
  --city "Munich" \
  --country "Germany" \
  --latitude 48.1353 \
  --longitude 11.5741 \
  --website "https://www.vfrischekaffee.de" \
  --created-at "2025-10-09T11:00:00Z"

# Madrid - Hola Coffee — Nov 2025
./target/debug/brewlog cafe add \
  --name "Hola Coffee" \
  --city "Madrid" \
  --country "Spain" \
  --latitude 40.4285 \
  --longitude -3.7025 \
  --website "https://www.holacoffee.es" \
  --created-at "2025-11-05T12:00:00Z"

# Madrid - Toma Café — Nov 2025
./target/debug/brewlog cafe add \
  --name "Toma Café" \
  --city "Madrid" \
  --country "Spain" \
  --latitude 40.4260 \
  --longitude -3.7075 \
  --created-at "2025-11-07T10:30:00Z"

# Bristol - Full Court Press — Jan 2026
./target/debug/brewlog cafe add \
  --name "Full Court Press" \
  --city "Bristol" \
  --country "United Kingdom" \
  --latitude 51.4543 \
  --longitude -2.5930 \
  --website "https://www.fullcourtpress.coffee" \
  --created-at "2026-01-11T10:00:00Z"

# Bristol - Small Street Espresso — Jan 2026
./target/debug/brewlog cafe add \
  --name "Small Street Espresso" \
  --city "Bristol" \
  --country "United Kingdom" \
  --latitude 51.4540 \
  --longitude -2.5955 \
  --created-at "2026-01-11T14:00:00Z"

# ============================================================================
# Cups - 1 to 3 cups per cafe, dated around cafe visits
# ============================================================================

# Monmouth Coffee (London, Aug 2025) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Red Brick Espresso") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Monmouth Coffee") | .id')" \
  --created-at "2025-08-12T11:30:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Kamwangi") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Monmouth Coffee") | .id')" \
  --created-at "2025-08-12T12:15:00Z"

# Prufrock Coffee (London, Aug 2025) - 3 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Kochere") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Prufrock Coffee") | .id')" \
  --created-at "2025-08-20T14:45:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="La Laja") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Prufrock Coffee") | .id')" \
  --created-at "2025-08-20T15:30:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Ben Saïd Natural") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Prufrock Coffee") | .id')" \
  --created-at "2025-08-25T10:00:00Z"

# The Barn (Berlin, Sep 2025) - 3 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Gatomboya") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="The Barn") | .id')" \
  --created-at "2025-09-08T10:30:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Los Pirineos") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="The Barn") | .id')" \
  --created-at "2025-09-08T15:00:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Halo Beriti") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="The Barn") | .id')" \
  --created-at "2025-09-09T09:30:00Z"

# Companion Coffee (Berlin, Sep 2025) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="El Paraiso 92 Anaerobic") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Companion Coffee") | .id')" \
  --created-at "2025-09-10T15:30:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Oreti SL28") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Companion Coffee") | .id')" \
  --created-at "2025-09-10T16:15:00Z"

# Man Versus Machine (Munich, Oct 2025) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Finca Tamana Washed") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Man Versus Machine") | .id')" \
  --created-at "2025-10-07T10:00:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="La Esmeralda Geisha") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Man Versus Machine") | .id')" \
  --created-at "2025-10-07T14:30:00Z"

# Vits der Kaffee (Munich, Oct 2025) - 1 cup
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="San Fermin") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Vits der Kaffee") | .id')" \
  --created-at "2025-10-09T11:30:00Z"

# Hola Coffee (Madrid, Nov 2025) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Cerro Azul") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Hola Coffee") | .id')" \
  --created-at "2025-11-05T12:30:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Guji Highland") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Hola Coffee") | .id')" \
  --created-at "2025-11-06T10:00:00Z"

# Toma Café (Madrid, Nov 2025) - 1 cup
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Daterra Sweet Collection") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Toma Café") | .id')" \
  --created-at "2025-11-07T11:00:00Z"

# Full Court Press (Bristol, Jan 2026) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Suke Quto") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Full Court Press") | .id')" \
  --created-at "2026-01-11T10:30:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Death to Decaf") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Full Court Press") | .id')" \
  --created-at "2026-01-11T11:15:00Z"

# Small Street Espresso (Bristol, Jan 2026) - 2 cups
./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Simbi") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Small Street Espresso") | .id')" \
  --created-at "2026-01-11T14:30:00Z"

./target/debug/brewlog cup add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="La Chumeca") | .id')" \
  --cafe-id "$(./target/debug/brewlog cafe list | jq -r '.[] | select(.name=="Small Street Espresso") | .id')" \
  --created-at "2026-01-11T15:00:00Z"

# ============================================================================
# Bags - Various bags from different roasters, spread over months 2-6
# Roast dates are slightly before created_at (bag bought after roasting)
# ============================================================================

# Bonanza Coffee - Gatomboya (175g) — Sep 2025, finished Oct
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Gatomboya") | .id')" \
  --roast-date "2025-09-20" \
  --amount 175 \
  --created-at "2025-09-28T10:00:00Z"

# Assembly Coffee - Kochere (250g) — early Oct, finished mid-Oct
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Kochere") | .id')" \
  --roast-date "2025-09-25" \
  --amount 250 \
  --created-at "2025-10-02T11:00:00Z"

# Coffee Collective - Daterra Sweet Collection (200g) — Oct, finished Nov
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Daterra Sweet Collection") | .id')" \
  --roast-date "2025-10-05" \
  --amount 200 \
  --created-at "2025-10-10T14:00:00Z"

# La Cabra - Halo Beriti (150g) — late Oct, finished Nov
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Halo Beriti") | .id')" \
  --roast-date "2025-10-18" \
  --amount 150 \
  --created-at "2025-10-25T09:00:00Z"

# Tim Wendelboe - Ben Saïd Natural (250g) — Nov, still open
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Ben Saïd Natural") | .id')" \
  --roast-date "2025-11-10" \
  --amount 250 \
  --created-at "2025-11-15T10:00:00Z"

# Tim Wendelboe - Finca Tamana Washed (350g) — Nov, still open
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Finca Tamana Washed") | .id')" \
  --roast-date "2025-11-15" \
  --amount 350 \
  --created-at "2025-11-20T13:00:00Z"

# Drop Coffee - La Linda (500g) — Dec, still open
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="La Linda") | .id')" \
  --roast-date "2025-12-05" \
  --amount 500 \
  --created-at "2025-12-10T11:00:00Z"

# April Coffee - Guji Highland (300g) — Dec, still open
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Guji Highland") | .id')" \
  --roast-date "2025-12-12" \
  --amount 300 \
  --created-at "2025-12-18T14:00:00Z"

# Square Mile Coffee - Red Brick Espresso (400g) — late Dec, still open
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Red Brick Espresso") | .id')" \
  --roast-date "2025-12-22" \
  --amount 400 \
  --created-at "2025-12-28T09:00:00Z"

# Dak Coffee Roasters - El Paraiso 92 Anaerobic (100g) — Jan 2026, still open
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="El Paraiso 92 Anaerobic") | .id')" \
  --roast-date "2026-01-05" \
  --amount 100 \
  --created-at "2026-01-10T10:00:00Z"

# Stow Coffee - Suke Quto (225g) — Jan 2026, still open
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Suke Quto") | .id')" \
  --roast-date "2026-01-10" \
  --amount 225 \
  --created-at "2026-01-15T12:00:00Z"

# Bows Coffee - Simbi (450g) — late Jan 2026, still open
./target/debug/brewlog bag add \
  --roast-id "$(./target/debug/brewlog roast list | jq -r '.[] | select(.name=="Simbi") | .id')" \
  --roast-date "2026-01-18" \
  --amount 450 \
  --created-at "2026-01-22T15:00:00Z"

# ============================================================================
# Finished Bags - Mark 4 oldest bags as finished
# ============================================================================

# Finish Gatomboya bag (Sep → Oct 2025)
./target/debug/brewlog bag update \
  --id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Gatomboya") | .id')" \
  --closed true \
  --finished-at "2025-10-15"

# Finish Kochere bag (Oct 2025)
./target/debug/brewlog bag update \
  --id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Kochere") | .id')" \
  --closed true \
  --finished-at "2025-10-20"

# Finish Daterra Sweet Collection bag (Oct → Nov 2025)
./target/debug/brewlog bag update \
  --id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Daterra Sweet Collection") | .id')" \
  --closed true \
  --finished-at "2025-11-05"

# Finish Halo Beriti bag (Oct → Nov 2025)
./target/debug/brewlog bag update \
  --id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Halo Beriti") | .id')" \
  --closed true \
  --finished-at "2025-11-12"

# ============================================================================
# Gear - 2 grinders, 3 brewers, and 3 filter papers
# Equipment added early (months 1-2), with upgrades in months 3-4
# ============================================================================

# Grinders
./target/debug/brewlog gear add \
  --category "grinder" \
  --make "Comandante" \
  --model "C40 MK4" \
  --created-at "2025-08-05T10:00:00Z"

./target/debug/brewlog gear add \
  --category "grinder" \
  --make "1Zpresso" \
  --model "J-Max" \
  --created-at "2025-09-20T11:00:00Z"

# Brewers
./target/debug/brewlog gear add \
  --category "brewer" \
  --make "Hario" \
  --model "V60 02" \
  --created-at "2025-08-10T14:30:00Z"

./target/debug/brewlog gear add \
  --category "brewer" \
  --make "AeroPress" \
  --model "Original" \
  --created-at "2025-08-15T09:00:00Z"

./target/debug/brewlog gear add \
  --category "brewer" \
  --make "Fellow" \
  --model "Stagg XF" \
  --created-at "2025-10-12T16:00:00Z"

# Filter Papers
./target/debug/brewlog gear add \
  --category "filter_paper" \
  --make "Hario" \
  --model "V60 Tabbed 02" \
  --created-at "2025-08-10T14:45:00Z"

./target/debug/brewlog gear add \
  --category "filter_paper" \
  --make "Sibarist" \
  --model "FAST Specialty 02" \
  --created-at "2025-10-01T09:00:00Z"

./target/debug/brewlog gear add \
  --category "filter_paper" \
  --make "Fellow" \
  --model "Stagg XF Filters" \
  --created-at "2025-10-12T16:15:00Z"

# ============================================================================
# Brews - Sample brews using open bags with realistic ratios (1:15 to 1:17)
# ============================================================================

# Standard V60 brew - Ben Saïd Natural with Comandante (ratio 1:16.7) — mid Nov
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Ben Saïd Natural") | .id')" \
  --coffee-weight 15.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="C40 MK4") | .id')" \
  --grind-setting 24.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 02") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 Tabbed 02") | .id')" \
  --water-volume 250 \
  --water-temp 92.0 \
  --quick-notes good \
  --brew-time 135 \
  --created-at "2025-11-18T08:30:00Z"

# AeroPress brew - Finca Tamana with J-Max (ratio 1:15) — late Nov
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Finca Tamana Washed") | .id')" \
  --coffee-weight 17.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="J-Max") | .id')" \
  --grind-setting 20.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Original") | .id')" \
  --water-volume 255 \
  --water-temp 88.0 \
  --brew-time 105 \
  --created-at "2025-11-25T09:00:00Z"

# Double V60 brew - La Linda with Comandante (ratio 1:16.7) — mid Dec
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="La Linda") | .id')" \
  --coffee-weight 30.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="C40 MK4") | .id')" \
  --grind-setting 25.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 02") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="FAST Specialty 02") | .id')" \
  --water-volume 500 \
  --water-temp 91.0 \
  --brew-time 165 \
  --created-at "2025-12-15T10:00:00Z"

# Stagg XF brew - Guji Highland with J-Max (ratio 1:16.7) — late Dec
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Guji Highland") | .id')" \
  --coffee-weight 18.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="J-Max") | .id')" \
  --grind-setting 22.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Stagg XF") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Stagg XF Filters") | .id')" \
  --water-volume 300 \
  --water-temp 93.0 \
  --quick-notes too-fast,under-extracted \
  --brew-time 110 \
  --created-at "2025-12-22T08:00:00Z"

# Light V60 brew - Red Brick Espresso with Comandante (ratio 1:16.7) — early Jan
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Red Brick Espresso") | .id')" \
  --coffee-weight 12.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="C40 MK4") | .id')" \
  --grind-setting 26.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 02") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 Tabbed 02") | .id')" \
  --water-volume 200 \
  --water-temp 94.0 \
  --brew-time 150 \
  --created-at "2026-01-02T09:30:00Z"

# AeroPress inverted - El Paraiso Anaerobic with J-Max (ratio 1:15) — mid Jan
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="El Paraiso 92 Anaerobic") | .id')" \
  --coffee-weight 15.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="J-Max") | .id')" \
  --grind-setting 18.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Original") | .id')" \
  --water-volume 225 \
  --water-temp 85.0 \
  --quick-notes too-hot,over-extracted \
  --brew-time 120 \
  --created-at "2026-01-12T08:15:00Z"

# V60 brew - Suke Quto with J-Max (ratio 1:16) — late Jan
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Suke Quto") | .id')" \
  --coffee-weight 20.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="J-Max") | .id')" \
  --grind-setting 21.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="V60 02") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="FAST Specialty 02") | .id')" \
  --water-volume 320 \
  --water-temp 92.0 \
  --quick-notes good \
  --brew-time 140 \
  --created-at "2026-01-20T09:00:00Z"

# Stagg XF brew - Simbi with Comandante (ratio 1:16) — late Jan
./target/debug/brewlog brew add \
  --bag-id "$(./target/debug/brewlog bag list | jq -r '.[] | select(.roast_name=="Simbi") | .id')" \
  --coffee-weight 16.0 \
  --grinder-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="C40 MK4") | .id')" \
  --grind-setting 23.0 \
  --brewer-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Stagg XF") | .id')" \
  --filter-paper-id "$(./target/debug/brewlog gear list | jq -r '.[] | select(.model=="Stagg XF Filters") | .id')" \
  --water-volume 256 \
  --water-temp 90.0 \
  --brew-time 130 \
  --created-at "2026-01-28T08:45:00Z"

# ============================================================================
# Enrich cafe timeline events with Position links
# (The Rust to_timeline_event() only includes City and Country)
# ============================================================================

DB_FILE="${DATABASE_URL:-brewlog.db}"
DB_FILE="${DB_FILE#sqlite://}"

sqlite3 "$DB_FILE" <<'ENDSQL'
.timeout 5000
UPDATE timeline_events SET details_json = (
  SELECT '[{"label":"City","value":"' || c.city || '"},{"label":"Country","value":"' || c.country || '"},{"label":"Website","value":"' || COALESCE(NULLIF(c.website, ''), '—') || '"},{"label":"Position","value":"https://www.google.com/maps?q=' || c.latitude || ',' || c.longitude || '"}]'
  FROM cafes c WHERE c.id = timeline_events.entity_id
)
WHERE entity_type = 'cafe';
ENDSQL

echo
echo "Bootstrapped database with timestamps set via --created-at"
echo
echo "Set token $BREWLOG_TOKEN to use the data added here."

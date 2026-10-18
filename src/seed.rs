use chrono::{Duration, TimeZone, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::products::model::Product;
use crate::products::repository;

pub const CATALOG_SIZE: usize = 100_000;

/// Wipe the products table and reload the deterministic catalog.
/// Running this any number of times leaves the database in the same state,
/// with product ids 1..=size matching the ids in `generate_large_catalog`.
pub async fn reset_and_seed(pool: &PgPool, size: usize) -> Result<(), sqlx::Error> {
    repository::truncate_products(pool).await?;
    let catalog = generate_large_catalog(size);
    repository::seed_products_bulk(pool, &catalog).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Vocabulary
// ---------------------------------------------------------------------------
struct Category {
    name: &'static str,
    nouns: &'static [&'static str],
    features: &'static [&'static str],
    min_cents: i64,
    max_cents: i64,
}

static CATEGORIES: [Category; 8] = [
    Category {
        name: "Electronics",
        nouns: &["keyboard", "mouse", "headphones", "speaker", "monitor", "webcam", "charger", "power bank", "router", "smartwatch"],
        features: &[
            "wireless bluetooth connectivity", "long battery life", "fast charging", "low latency for gaming",
            "active noise cancelling", "ergonomic design", "rgb backlighting", "usb-c connection",
            "crisp high resolution output", "plug and play setup",
        ],
        min_cents: 1_999,
        max_cents: 39_999,
    },
    Category {
        name: "Footwear",
        nouns: &["running shoes", "trail shoes", "hiking boots", "winter boots", "sandals", "sneakers", "slippers", "rain boots"],
        features: &[
            "cushioned sole", "breathable mesh upper", "insulated lining that keeps feet warm", "waterproof membrane",
            "slip resistant grip", "lightweight and flexible construction", "extra arch support", "reflective details for night visibility",
        ],
        min_cents: 3_999,
        max_cents: 24_999,
    },
    Category {
        name: "Clothing",
        nouns: &["jacket", "parka", "hoodie", "t-shirt", "fleece", "gloves", "beanie", "jeans", "socks"],
        features: &[
            "windproof outer shell", "thermal insulation for cold weather", "moisture wicking fabric", "machine washable",
            "quick drying material", "soft brushed lining", "reinforced stitching", "adjustable hood and cuffs",
        ],
        min_cents: 1_499,
        max_cents: 29_999,
    },
    Category {
        name: "Outdoor",
        nouns: &["tent", "backpack", "sleeping bag", "camp stove", "headlamp", "trekking poles", "cooler", "hammock"],
        features: &[
            "weatherproof construction", "packs down small", "easy setup in minutes", "tear resistant ripstop fabric",
            "bright long lasting light", "large main compartment", "padded shoulder straps", "rated for freezing temperatures",
        ],
        min_cents: 1_999,
        max_cents: 49_999,
    },
    Category {
        name: "Kitchen",
        nouns: &["coffee maker", "kettle", "blender", "air fryer", "knife set", "cutting board", "toaster", "water bottle"],
        features: &[
            "brews fresh coffee in minutes", "keeps drinks cold for hours", "dishwasher safe parts", "powerful motor",
            "non stick coating", "stainless steel body", "programmable timer", "compact footprint for small counters",
        ],
        min_cents: 1_299,
        max_cents: 24_999,
    },
    Category {
        name: "Home",
        nouns: &["desk lamp", "office chair", "standing desk", "bookshelf", "throw blanket", "humidifier", "space heater", "curtains"],
        features: &[
            "adjustable brightness", "quiet operation", "sturdy solid frame", "soft touch fabric",
            "energy efficient design", "tool free assembly", "warms a room quickly", "blocks out light",
        ],
        min_cents: 1_999,
        max_cents: 59_999,
    },
    Category {
        name: "Fitness",
        nouns: &["yoga mat", "dumbbell set", "resistance bands", "jump rope", "foam roller", "gym bag"],
        features: &[
            "non slip surface", "adjustable weight", "durable rubber construction", "easy to store",
            "supports recovery after workouts", "suitable for beginners and athletes", "sweat resistant material",
        ],
        min_cents: 999,
        max_cents: 19_999,
    },
    Category {
        name: "Accessories",
        nouns: &["wallet", "sunglasses", "watch", "belt", "umbrella", "laptop bag", "key holder"],
        features: &[
            "genuine leather finish", "uv protection lenses", "slim profile fits any pocket", "scratch resistant surface",
            "compact and foldable", "secure zipper closure", "water resistant design",
        ],
        min_cents: 999,
        max_cents: 34_999,
    },
];

// made-up brands, no real companies
static BRANDS: [&str; 24] = [
    "Nordvik", "Aerolite", "Kestrel", "Brightwave", "Ironpeak", "Solace", "Tundra", "Zenith",
    "Orbital", "Fernhill", "Copperline", "Halcyon", "Voltaic", "Driftwood", "Summit", "Lumen",
    "Pinecrest", "Quartz", "Rivet", "Stellar", "Maple", "Granite", "Cobalt", "Willow",
];

static ADJECTIVES: [&str; 30] = [
    "lightweight", "durable", "compact", "premium", "ergonomic", "portable", "waterproof", "insulated",
    "adjustable", "reusable", "wireless", "smart", "heavy-duty", "foldable", "rechargeable", "breathable",
    "classic", "modern", "rugged", "ultralight", "stainless", "quiet", "fast", "versatile",
    "eco-friendly", "all-weather", "professional", "everyday", "budget", "deluxe",
];

static USES: [&str; 10] = [
    "everyday use", "long trips", "cold winters", "busy mornings", "outdoor adventures",
    "the home office", "gym sessions", "daily commuting", "camping weekends", "small spaces",
];

static EXTRAS: [&str; 8] = [
    "Backed by a two year warranty.",
    "Available in several colors.",
    "Easy to clean and maintain.",
    "A great gift idea.",
    "Made with recycled materials.",
    "Ships ready to use.",
    "Trusted by thousands of customers.",
    "Designed with feedback from real users.",
];

/// Scrambles a product number into an index between 0 and len-1.
/// The same (i, salt) always gives the same answer, so the catalog never changes between runs,
/// but every field gets an unrelated choice. Use a different `salt` for each field.
/// (The extra shifting and multiplying "avalanches" the bits: a tiny change in `i` or `salt`
/// changes the whole result. Without it, different fields end up correlated.)
fn choose(i: usize, salt: u64, len: usize) -> usize {
    let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z % len as u64) as usize
}

/// Builds the same catalog every time (no randomness, no `Utc::now()`), so benchmarks are repeatable.
/// A catalog of 1,000 products is exactly the first 1,000 products of a catalog of 100,000.
pub fn generate_large_catalog(size: usize) -> Vec<Product> {
    let mut products = Vec::with_capacity(size);
    let base_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

    for i in 1..=size {
        let category = &CATEGORIES[choose(i, 1, CATEGORIES.len())];
        let noun = category.nouns[choose(i, 2, category.nouns.len())];
        let brand = BRANDS[choose(i, 3, BRANDS.len())];
        let adjective = ADJECTIVES[choose(i, 4, ADJECTIVES.len())];
        let use_case = USES[choose(i, 5, USES.len())];

        let title = format!("{} {} {} M{}", brand, adjective, noun, 100 + choose(i, 6, 900));

        // 1-3 features and 0-2 extra sentences, so descriptions differ in length
        // (that is what keeps BM25 scores from tying)
        let feature_count = 1 + choose(i, 7, 3);
        let feature_start = choose(i, 8, category.features.len());
        let features: Vec<&str> = (0..feature_count)
            .map(|k| category.features[(feature_start + k * 3) % category.features.len()])
            .collect();

        let mut description = format!(
            "{} {} from {}, built for {}. Features {}.",
            adjective,
            noun,
            brand,
            use_case,
            features.join(", ")
        );

        let extra_count = choose(i, 9, 3);
        let extra_start = choose(i, 10, EXTRAS.len());
        for k in 0..extra_count {
            description.push(' ');
            description.push_str(EXTRAS[(extra_start + k * 3) % EXTRAS.len()]);
        }

        let span = (category.max_cents - category.min_cents + 1) as usize;
        let cents = category.min_cents + choose(i, 11, span) as i64;

        products.push(Product {
            id: i as i32,
            title,
            description,
            category: category.name.to_string(),
            price: Decimal::new(cents, 2),
            in_stock: choose(i, 12, 10) != 0, // about 10% out of stock
            updated_at: base_time + Duration::minutes(choose(i, 13, 525_600) as i64), // within one year
        });
    }

    products
}

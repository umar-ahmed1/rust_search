#[derive(Debug)]
pub struct Product {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug)]
pub struct SearchResult<'a> {
    pub product: &'a Product,
    pub score: f64,
}

pub fn sample_products() -> Vec<Product> {
    vec![
        Product {
            id: 1,
            title: "Running Shoes".to_string(),
            description: "Lightweight shoes designed for everyday running".to_string(),
            category: "Footwear".to_string(),
        },
        Product {
            id: 2,
            title: "Hiking Boots".to_string(),
            description: "Waterproof boots for hiking and mountain trails".to_string(),
            category: "Footwear".to_string(),
        },
        Product {
            id: 3,
            title: "Trail Running Shoes".to_string(),
            description: "Durable shoes for running on rough terrain".to_string(),
            category: "Footwear".to_string(),
        },
        Product {
            id: 4,
            title: "Running Jacket".to_string(),
            description: "Lightweight waterproof jacket for outdoor exercise".to_string(),
            category: "Clothing".to_string(),
        },
        Product {
            id: 5,
            title: "Cotton T-Shirt".to_string(),
            description: "Comfortable everyday shirt made from soft cotton".to_string(),
            category: "Clothing".to_string(),
        },
        Product {
            id: 6,
            title: "Winter Jacket".to_string(),
            description: "Insulated jacket designed for cold weather".to_string(),
            category: "Clothing".to_string(),
        },
        Product {
            id: 7,
            title: "Gaming Keyboard".to_string(),
            description: "Mechanical keyboard with responsive switches".to_string(),
            category: "Electronics".to_string(),
        },
        Product {
            id: 8,
            title: "Wireless Mouse".to_string(),
            description: "Ergonomic wireless mouse for gaming and productivity".to_string(),
            category: "Electronics".to_string(),
        },
        Product {
            id: 9,
            title: "Gaming Headphones".to_string(),
            description: "Wireless headphones with surround sound".to_string(),
            category: "Electronics".to_string(),
        },
        Product {
            id: 10,
            title: "Laptop".to_string(),
            description: "Portable computer with a high performance processor".to_string(),
            category: "Electronics".to_string(),
        },
        Product {
            id: 11,
            title: "Coffee Maker".to_string(),
            description: "Automatic coffee machine for brewing fresh coffee".to_string(),
            category: "Home".to_string(),
        },
        Product {
            id: 12,
            title: "Water Bottle".to_string(),
            description: "Reusable insulated bottle that keeps drinks cold".to_string(),
            category: "Outdoor".to_string(),
        },
        Product {
            id: 13,
            title: "Camping Tent".to_string(),
            description: "Lightweight waterproof tent for outdoor camping".to_string(),
            category: "Outdoor".to_string(),
        },
        Product {
            id: 14,
            title: "Hiking Backpack".to_string(),
            description: "Durable backpack with storage for long hikes".to_string(),
            category: "Outdoor".to_string(),
        },
        Product {
            id: 15,
            title: "Desk Lamp".to_string(),
            description: "Adjustable LED lamp for reading and studying".to_string(),
            category: "Home".to_string(),
        },
    ]
}

// rust could be confused if the result is pointing to products or query so use 'a to make it aware
pub fn search_products<'a>(products: &'a Vec<Product>, query: &str) -> Vec<SearchResult<'a>> {
    //normalize the query
    let lowercase: String = query.to_lowercase();

    // split the query into words ex: "trail running shoes" -> ["trail", "running", "shoes"]
    let filters: Vec<&str> = lowercase.split_whitespace().collect();

    //if there are no filters then return empty vector
    if filters.is_empty() {
        return vec![];
    }

    let mut results: Vec<SearchResult> = Vec::new();

    //perform the search and return the products
    for product in products {
        // normalize the title/description
        let title: String = product.title.to_lowercase();
        let description: String = product.description.to_lowercase();

        //search through the filters and find the product -> for now if any word matches we can return the product so we r doing an OR over filters
        let matches_title = filters.iter().any(|filter| title.contains(filter));
        let matches_description = filters.iter().any(|filter| description.contains(filter));

        if !(matches_title || matches_description) {
            continue;
        }

        let mut score: f64 = 0.0;
        if matches_title { score += 10.0};
        if matches_description { score += 5.0};

        let search_result = SearchResult {
            product, 
            score
        };

        results.push(search_result);
    }
    // sort search results by highest score first
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let products = sample_products();
        let results = search_products(&products, "");
        assert!(results.is_empty(), "Empty query should return 0 results");
    }

    #[test]
    fn test_case_insensitivity() {
        let products = sample_products();
        // Search in all caps
        let results = search_products(&products, "RUNNING SHOES");
        assert!(!results.is_empty(), "Case-insensitive search should return results");
    }

    #[test]
    fn test_missing_terms() {
        let products = sample_products();
        let results = search_products(&products, "spaceship laser gun");
        assert!(results.is_empty(), "Non-existent terms should return 0 results");
    }

    #[test]
    fn test_or_logic() {
        let products = sample_products();
        // "hiking jacket" contains two terms. With OR logic, a product 
        // matching *either* "hiking" or "jacket" should be returned.
        let results = search_products(&products, "hiking jacket");

        // Verify we get matches for hiking items AND jacket items
        let matched_ids: Vec<i32> = results.iter().map(|r| r.product.id).collect();
        
        // Product 2 (Hiking Boots) should match "hiking"
        // Product 4 (Running Jacket) should match "jacket"
        assert!(matched_ids.contains(&2));
        assert!(matched_ids.contains(&4));
    }

    #[test]
    fn test_ranking_sort_order() {
        let products = sample_products();
        let results = search_products(&products, "running");

        // Verify results are sorted from highest score to lowest score
        for i in 0..results.len().saturating_sub(1) {
            assert!(
                results[i].score >= results[i + 1].score,
                "Results are not sorted correctly by score!"
            );
        }
    }
}
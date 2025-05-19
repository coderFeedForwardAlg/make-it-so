
        CREATE TABLE public."user" (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            favorite_color VARCHAR(50),
            height NUMERIC,
            age INTEGER,
            job VARCHAR(100)
        );

        CREATE TABLE product_details (
            product_id INTEGER PRIMARY KEY,
            description TEXT,
            price DECIMAL(10, 2)
        );


        create table order_items (
            order_id INTEGER,
            item_id INTEGER,
            quantity INTEGER
        );
        
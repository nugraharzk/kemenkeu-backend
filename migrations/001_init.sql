CREATE DATABASE IF NOT EXISTS kemenkeu CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE kemenkeu;

CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    monthly_budget_cents BIGINT NOT NULL DEFAULT 0
);

INSERT INTO users (name, monthly_budget_cents) VALUES ('Ibu', 7000000), ('Papa', 1700000), ('Admin', 0);

CREATE TABLE categories (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    type VARCHAR(10) NOT NULL DEFAULT 'expense',
    icon VARCHAR(30) NOT NULL DEFAULT 'more',
    budget_percent INT DEFAULT 0,
    UNIQUE KEY uq_cat_name_type (name, type)
);

INSERT INTO categories (name, type, icon) VALUES
    ('Gaji', 'income', 'money'),
    ('Bonus', 'income', 'gift'),
    ('Investasi', 'income', 'trending-up'),
    ('Lainnya', 'income', 'wallet-add'),
    ('Makan', 'expense', 'coffee'),
    ('Transport', 'expense', 'car'),
    ('Tagihan', 'expense', 'document'),
    ('Belanja', 'expense', 'shopping-cart'),
    ('Hiburan', 'expense', 'game'),
    ('Pendidikan', 'expense', 'book'),
    ('Kesehatan', 'expense', 'heart'),
    ('Tabungan', 'expense', 'save-2'),
    ('Lainnya', 'expense', 'more')
ON DUPLICATE KEY UPDATE icon = VALUES(icon);

CREATE TABLE transactions (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    amount_cents BIGINT NOT NULL,
    category_id INT NOT NULL,
    note VARCHAR(255) DEFAULT '',
    date DATE NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

CREATE INDEX idx_tx_user ON transactions(user_id);
CREATE INDEX idx_tx_date ON transactions(date);
CREATE INDEX idx_tx_category ON transactions(category_id);

CREATE TABLE budgets (
    id INT AUTO_INCREMENT PRIMARY KEY,
    category_id INT NOT NULL,
    person VARCHAR(20) DEFAULT NULL,
    month DATE NOT NULL,
    amount_cents BIGINT NOT NULL,
    UNIQUE KEY uq_budget (category_id, person, month),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

CREATE INDEX idx_budget_month ON budgets(month);
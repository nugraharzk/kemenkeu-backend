CREATE DATABASE IF NOT EXISTS kemenkeu CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE kemenkeu;

CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE
);

INSERT INTO users (name) VALUES ('Saya'), ('Pasangan');

CREATE TABLE categories (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    type VARCHAR(10) NOT NULL DEFAULT 'expense',
    icon VARCHAR(10) NOT NULL DEFAULT '📦',
    UNIQUE KEY uq_cat_name_type (name, type)
);

INSERT INTO categories (name, type, icon) VALUES
    ('Gaji', 'income', '💵'),
    ('Bonus', 'income', '🎁'),
    ('Investasi', 'income', '📈'),
    ('Lainnya', 'income', '💰'),
    ('Makan', 'expense', '🍽️'),
    ('Transport', 'expense', '🚗'),
    ('Tagihan', 'expense', '📄'),
    ('Belanja', 'expense', '🛒'),
    ('Hiburan', 'expense', '🎮'),
    ('Pendidikan', 'expense', '📚'),
    ('Kesehatan', 'expense', '💊'),
    ('Tabungan', 'expense', '🏦'),
    ('Lainnya', 'expense', '📦');

CREATE TABLE transactions (
    id INT AUTO_INCREMENT PRIMARY KEY,
    person VARCHAR(20) NOT NULL,
    amount_cents BIGINT NOT NULL,
    category_id INT NOT NULL,
    note VARCHAR(255) DEFAULT '',
    date DATE NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

CREATE INDEX idx_tx_person ON transactions(person);
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
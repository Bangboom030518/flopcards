BEGIN TRANSACTION;

CREATE TABLE cardset (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  created DATETIME DEFAULT CURRENT_TIMESTAMP,
  subject TEXT NOT NULL,
  FOREIGN KEY (subject) REFERENCES subject (id)
);

CREATE TABLE folder (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  created DATETIME DEFAULT CURRENT_TIMESTAMP,
  subject TEXT NOT NULL,
  FOREIGN KEY (subject) REFERENCES subject (id)
);

CREATE TABLE subject (
  id TEXT PRIMARY KEY,
  color TEXT NOT NULL
);

INSERT INTO subject (id, color) VALUES
('maths', 'red'),
('geography', 'emerald'),
('other', 'purple'),
('spanish', 'yellow');

CREATE TABLE term (
  id INTEGER PRIMARY KEY,
  term TEXT NOT NULL,
  definition TEXT NOT NULL,
  created DATETIME DEFAULT CURRENT_TIMESTAMP,
  cardset INTEGER NOT NULL,
  FOREIGN KEY (cardset) REFERENCES cardset (id)
);

COMMIT;

-- # vim: tabstop=4 shiftwidth=4

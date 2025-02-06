BEGIN TRANSACTION;

CREATE TABLE cardset (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  parent TEXT,
  created DATETIME DEFAULT CURRENT_TIMESTAMP,
  subject TEXT NOT NULL,
  FOREIGN KEY (subject) REFERENCES subject (id),
  FOREIGN KEY (parent) REFERENCES folder (id)
);

CREATE TABLE folder (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  created DATETIME DEFAULT CURRENT_TIMESTAMP,
  subject TEXT NOT NULL,
  FOREIGN KEY (subject) REFERENCES subject (id)
);

CREATE TABLE subject (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL, 
  color TEXT NOT NULL
);

INSERT INTO subject (id, name, color) VALUES
('maths', 'maths', 'red'),
('geography', 'geography', 'emerald'),
('other', 'other', 'purple'),
('spanish', 'spanish', 'yellow');

CREATE TABLE card (
  id INTEGER PRIMARY KEY,
  term TEXT NOT NULL,
  definition TEXT NOT NULL,
  cardset INTEGER NOT NULL,
  FOREIGN KEY (cardset) REFERENCES cardset (id)
);

COMMIT;

-- # vim: tabstop=2 shiftwidth=2

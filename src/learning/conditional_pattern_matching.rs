struct User {
  name: String,
  age: u32,
  role: String,
}

fn process_user(user: &User) {
  match user {
      User { age, role, .. } if age > &21 && role == "admin" => {
          println!("Adult administrator: full access granted");
      }
      User { age, role, .. } if age > &21 && role == "user" => {
          println!("Adult user: standard access granted");
      }
      User { age, .. } if age > &13 => {
          println!("Teen user: restricted access granted");
      }
      _ => println!("Access denied"),
  }
}
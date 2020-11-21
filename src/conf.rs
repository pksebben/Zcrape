pub struct Conf {
    pub user: String,
    pub key: String,
}

impl Conf{
    pub fn new() -> Conf{
	Conf{
	    user : String::from("benmorsillo@gmail.com"),
	    key : String::from("8IELo8VZoanuCs4YRGOl1Op0qfoXq6DH"),
	}
    }
    pub fn user(self) -> String{
	self.user
    }

    pub fn key(self) -> String{
	self.key
    }
}

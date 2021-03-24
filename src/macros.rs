macro_rules! req {
    ($path:expr, $type:ident, $self:ident, $body:ident, $do:stmt) => {
        if let Ok(response) = $self
            .client
            .$type(&format!("{}/{}", $self.server_url, $path))
            .send()
            .await
        {
            if let Ok($body) = response.text().await {
                if let Ok(error) = serde_json::from_str::<ApiError>(&$body) {
                    return Err(error.into());
                }
                $do
            } else {
                Err(crate::result::Error::UnexpectedResponse)
            }
        } else {
            Err(crate::result::Error::Connection)
        }
    };
    ($path:expr, $type:ident, $self:ident) => {
        if let Ok(response) = $self
            .client
            .$type(&format!("{}/{}", $self.server_url, $path))
            .send()
            .await
        {
            if response.status().is_success() {
                return Ok(());
            }
            if let Ok(body) = response.text().await {
                if let Ok(error) = serde_json::from_str::<ApiError>(&body) {
                    Err(error.into())
                } else {
                    Err(crate::result::Error::UnexpectedResponse)
                }
            } else {
                Err(crate::result::Error::UnexpectedResponse)
            }
        } else {
            Err(crate::result::Error::Connection)
        }
    };
}

macro_rules! get {
    ($path:expr, $type:ident, $self:ident) => {
        req!(
            $path,
            get,
            $self,
            body,
            return serde_json::from_str::<$type>(&body)
                .map_err(|_| crate::result::Error::UnexpectedResponse)
        )
    };
    ($path:expr, $self:ident) => {
        req!($path, get, $self)
    };
}

macro_rules! post {
    ($path:expr, $type:ident, $self:ident) => {
        req!(
            $path,
            post,
            $self,
            body,
            return $type::from_str(&body).map_err(|_| crate::result::Error::UnexpectedResponse)
        )
    };
    ($path:expr, $self:ident) => {
        req!($path, post, $self)
    };
}

macro_rules! put {
    ($path:expr, $type:ident, $self:ident) => {
        req!(
            $path,
            put,
            $self,
            body,
            return $type::from_str(&body).map_err(|_| crate::result::Error::UnexpectedResponse)
        )
    };
    ($path:expr, $self:ident) => {
        req!($path, put, $self)
    };
}

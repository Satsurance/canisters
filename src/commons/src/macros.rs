#[macro_export]
macro_rules! canister_methods {
    (
        $(
            update $update_name:ident($($update_param:ident: $update_param_type:ty),*) -> $update_return:ty;
        )*
        $(
            query $query_name:ident($($query_param:ident: $query_param_type:ty),*) -> $query_return:ty;
        )*
    ) => {
        $(
            crate::__impl_update_method!(self, $update_name, $update_return, $($update_param: $update_param_type),*);
        )*
        $(
            crate::__impl_query_method!(self, $query_name, $query_return, $($query_param: $query_param_type),*);
        )*
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_update_method {
    ($self:ident, $name:ident, $return:ty, $param1:ident: $param1_type:ty, $($param:ident: $param_type:ty),+) => {
        pub fn $name(&$self, $param1: $param1_type, $($param: $param_type),+) -> $return {
            $self.client.update(stringify!($name), ($param1, $($param),+))
                .expect(&format!("Failed to call {}", stringify!($name)))
        }
    };
 
    ($self:ident, $name:ident, $return:ty, $param:ident: $param_type:ty) => {
        pub fn $name(&$self, $param: $param_type) -> $return {
            $self.client.update(stringify!($name), ($param,))
                .expect(&format!("Failed to call {}", stringify!($name)))
        }
    };
  
    ($self:ident, $name:ident, $return:ty, ) => {
        pub fn $name(&$self) -> $return {
            $self.client.update_no_args(stringify!($name))
                .expect(&format!("Failed to call {}", stringify!($name)))
        }
    };
}


#[macro_export]
#[doc(hidden)]
macro_rules! __impl_query_method {
    ($self:ident, $name:ident, $return:ty, $param1:ident: $param1_type:ty, $($param:ident: $param_type:ty),+) => {
        pub fn $name(&$self, $param1: $param1_type, $($param: $param_type),+) -> $return {
            $self.client.query(stringify!($name), ($param1, $($param),+))
                .expect(&format!("Failed to call {}", stringify!($name)))
        }
    };
    ($self:ident, $name:ident, $return:ty, $param:ident: $param_type:ty) => {
        pub fn $name(&$self, $param: $param_type) -> $return {
            $self.client.query(stringify!($name), ($param,))
                .expect(&format!("Failed to call {}", stringify!($name)))
        }
    };
    ($self:ident, $name:ident, $return:ty, ) => {
        pub fn $name(&$self) -> $return {
            $self.client.query_no_args(stringify!($name))
                .expect(&format!("Failed to call {}", stringify!($name)))
        }
    };
}



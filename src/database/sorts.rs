use crate::ServerError;

pub trait Sortable {
    /// Must be implemented- get the allowed fields to sort by (eg: [ "name"])
    fn get_allowed_fields() -> &'static [&'static str];

    /// Optionally implemented, allows a default sorting (to replace None)
    fn default_sort() -> Option<&'static str> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct SortableColumn<T : Sortable> {
    column: Option<String>,
    sort_order: SortOrder,
    _marker: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl<T: Sortable> SortableColumn<T> {
    pub fn get_column(&self) -> Option<String> {
        self.column.clone()
    }

    pub fn get_sort_direction_i32(&self) -> i32 {
        match self.sort_order {
            SortOrder::Ascending => 1,
            SortOrder::Descending => -1,
        }
    }

    pub fn try_parse(
        column: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<SortableColumn<T>, ServerError> {
        let column = column.or(T::default_sort());
        
        let allowed_asc = ["asc", "ascending", "a", "1", "true"];
        let allowed_desc = ["desc", "descending", "d", "-1", "false"];

        let sort_order = if let Some(order) = sort_order {
            if allowed_asc.contains(&order) {
                SortOrder::Ascending
            } else if allowed_desc.contains(&order) {
                SortOrder::Descending
            } else {
                return Err(ServerError::BadRequest(format!(
                    "Invalid sort order: {}. Allowed values: {:?}",
                    order,
                    [allowed_asc, allowed_desc].concat()
                )));
            }
        } else {
            SortOrder::Ascending // Default to ascending if no order is provided
        };

        if let Some(ref column) = column {       
            if T::get_allowed_fields().contains(column) {
                Ok(SortableColumn::<T> {
                    column: Some(column.to_string()),
                    sort_order,
                    _marker: std::marker::PhantomData,
                })
            } else {
                Err(ServerError::BadRequest(format!(
                    "Invalid sort column: {}. Allowed columns: {:?}",
                    column,
                    T::get_allowed_fields()
                )))
            }
        } else {
            Ok(SortableColumn::<T> {
                column: None,
                sort_order,
                _marker: std::marker::PhantomData,
            })
        }
    }
}




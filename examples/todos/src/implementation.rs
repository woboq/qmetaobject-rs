/*
 *   Based on an example from rust-qt-binding-generator
 *   Copyright 2017  Jos van den Oever <jos@vandenoever.info>
 *
 *   This program is free software; you can redistribute it and/or
 *   modify it under the terms of the GNU General Public License as
 *   published by the Free Software Foundation; either version 2 of
 *   the License or (at your option) version 3 or any later version
 *   accepted by the membership of KDE e.V. (or its successor approved
 *   by the membership of KDE e.V.), which shall act as a proxy
 *   defined in Section 14 of version 3 of the license.
 *
 *   This program is distributed in the hope that it will be useful,
 *   but WITHOUT ANY WARRANTY; without even the implied warranty of
 *   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *   GNU General Public License for more details.
 *
 *   You should have received a copy of the GNU General Public License
 *   along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use qmetaobject::*;
use std::collections::HashMap;

#[derive(Default, Clone)]
struct TodosItem {
    completed: bool,
    description: String,
}

#[allow(non_snake_case)]
#[derive(Default, QObject)]
pub struct Todos {
    base: qt_base_class!(trait QAbstractListModel),
    count: qt_property!(i32; READ row_count NOTIFY count_changed),
    count_changed: qt_signal!(),
    list: Vec<TodosItem>,
    activeCount: qt_property!(usize; NOTIFY active_count_changed),
    active_count_changed: qt_signal!(),

    setCompleted: qt_method!(fn(&mut self, item: usize, v: bool) -> bool),
    setDescription: qt_method!(fn(&mut self, item: usize, v: String) -> bool ),
    insert_rows: qt_method!(fn(&mut self, row: usize, count: usize) -> bool),
    remove_rows: qt_method!(fn(&mut self, row: usize, count: usize) -> bool),
    clearCompleted: qt_method!(fn(&mut self)),
    add: qt_method!(fn(&mut self, description: String)),
    remove: qt_method!(fn(&mut self, index: u64) -> bool),
    setAll: qt_method!(fn(&mut self, completed: bool)),
}

impl Todos {
    fn update_active_count(&mut self) {
        let ac = self.list.iter().filter(|i| !i.completed).count();
        if self.activeCount != ac {
            self.activeCount = ac;
            self.active_count_changed();
        }
    }

    #[allow(non_snake_case)]
    fn setCompleted(&mut self, item: usize, v: bool) -> bool {
        if item >= self.list.len() {
            return false;
        }
        self.list[item].completed = v;
        let idx = (self as &mut dyn QAbstractListModel).row_index(item as i32);
        (self as &mut dyn QAbstractListModel).data_changed(idx.clone(), idx);
        self.update_active_count();
        true
    }

    #[allow(non_snake_case)]
    fn setDescription(&mut self, item: usize, v: String) -> bool {
        if item >= self.list.len() {
            return false;
        }
        self.list[item].description = v;
        let idx = (self as &mut dyn QAbstractListModel).row_index(item as i32);
        (self as &mut dyn QAbstractListModel).data_changed(idx.clone(), idx);
        true
    }

    fn insert_rows(&mut self, row: usize, count: usize) -> bool {
        if count == 0 || row > self.list.len() {
            return false;
        }
        (self as &mut dyn QAbstractListModel)
            .begin_insert_rows(row as i32, (row + count - 1) as i32);
        for i in 0..count {
            self.list.insert(row + i, TodosItem::default());
        }
        (self as &mut dyn QAbstractListModel).end_insert_rows();
        self.activeCount += count;
        self.active_count_changed();
        self.count_changed();
        true
    }

    fn remove_rows(&mut self, row: usize, count: usize) -> bool {
        if count == 0 || row + count > self.list.len() {
            return false;
        }
        (self as &mut dyn QAbstractListModel)
            .begin_remove_rows(row as i32, (row + count - 1) as i32);
        self.list.drain(row..row + count);
        (self as &mut dyn QAbstractListModel).end_remove_rows();
        self.count_changed();
        self.update_active_count();
        true
    }

    #[allow(non_snake_case)]
    fn clearCompleted(&mut self) {
        (self as &mut dyn QAbstractListModel).begin_reset_model();
        self.list.retain(|i| !i.completed);
        (self as &mut dyn QAbstractListModel).end_reset_model();
        self.count_changed();
    }

    fn add(&mut self, description: String) {
        let end = self.list.len();
        (self as &mut dyn QAbstractListModel).begin_insert_rows(end as i32, end as i32);
        self.list.insert(end, TodosItem { completed: false, description });
        (self as &mut dyn QAbstractListModel).end_insert_rows();
        self.activeCount += 1;
        self.active_count_changed();
        self.count_changed();
    }

    fn remove(&mut self, index: u64) -> bool {
        self.remove_rows(index as usize, 1)
    }

    #[allow(non_snake_case)]
    fn setAll(&mut self, completed: bool) {
        for i in &mut self.list {
            i.completed = completed;
        }

        let idx1 = (self as &mut dyn QAbstractListModel).row_index(0);
        let end = self.list.len() as i32;
        let idx2 = (self as &mut dyn QAbstractListModel).row_index(end - 1);
        (self as &mut dyn QAbstractListModel).data_changed(idx1, idx2);
        self.update_active_count();
    }
}

impl QAbstractListModel for Todos {
    fn row_count(&self) -> i32 {
        self.list.len() as i32
    }
    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let idx = index.row() as usize;
        if idx < self.list.len() {
            if role == USER_ROLE {
                self.list[idx].completed.into()
            } else if role == USER_ROLE + 1 {
                QString::from(self.list[idx].description.clone()).into()
            } else {
                QVariant::default()
            }
        } else {
            QVariant::default()
        }
    }
    fn role_names(&self) -> HashMap<i32, QByteArray> {
        let mut map = HashMap::new();
        map.insert(USER_ROLE, "completed".into());
        map.insert(USER_ROLE + 1, "description".into());
        map
    }
}

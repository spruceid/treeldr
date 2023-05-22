pub mod rdfs {
    pub mod trait_object {
        pub struct ResourceDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for ResourceDynTable<C> {
            type Instance < 'a > = ResourceDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct ResourceDynTableInstance<'a, C: ?Sized> {
            pub type_: unsafe fn(
                *const u8,
                context: ::treeldr_rust_prelude::ContravariantReference<'a, C>,
            ) -> ::treeldr_rust_prelude::BoxedDynIterator<
                DynResourceType<'a, C>,
            >,
            pub label: unsafe fn(
                *const u8,
                context: ::treeldr_rust_prelude::ContravariantReference<'a, C>,
            ) -> ::treeldr_rust_prelude::BoxedDynIterator<
                DynResourceLabel<'a, C>,
            >,
            pub comment: unsafe fn(
                *const u8,
                context: ::treeldr_rust_prelude::ContravariantReference<'a, C>,
            ) -> ::treeldr_rust_prelude::BoxedDynIterator<
                DynResourceComment<'a, C>,
            >,
            pub see_also: unsafe fn(
                *const u8,
                context: ::treeldr_rust_prelude::ContravariantReference<'a, C>,
            ) -> ::treeldr_rust_prelude::BoxedDynIterator<
                DynResourceSeeAlso<'a, C>,
            >,
            pub is_defined_by: unsafe fn(
                *const u8,
                context: ::treeldr_rust_prelude::ContravariantReference<'a, C>,
            ) -> ::treeldr_rust_prelude::BoxedDynIterator<
                DynResourceIsDefinedBy<'a, C>,
            >,
        }
        impl<'a, C: ?Sized> Clone for ResourceDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for ResourceDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> ResourceDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Resource<C>>() -> Self {
                Self {
                    type_: |ptr, context| unsafe {
                        let subject = &*(ptr as *const T);
                        let object = context.get(|context| subject.type_(context));
                        ::treeldr_rust_prelude::BoxedDynIterator::new(
                            object.map(DynResourceType::<'a, C>::new),
                        )
                    },
                    label: |ptr, context| unsafe {
                        let subject = &*(ptr as *const T);
                        let object = context.get(|context| subject.label(context));
                        ::treeldr_rust_prelude::BoxedDynIterator::new(
                            object.map(DynResourceLabel::<'a, C>::new),
                        )
                    },
                    comment: |ptr, context| unsafe {
                        let subject = &*(ptr as *const T);
                        let object = context.get(|context| subject.comment(context));
                        ::treeldr_rust_prelude::BoxedDynIterator::new(
                            object.map(DynResourceComment::<'a, C>::new),
                        )
                    },
                    see_also: |ptr, context| unsafe {
                        let subject = &*(ptr as *const T);
                        let object = context.get(|context| subject.see_also(context));
                        ::treeldr_rust_prelude::BoxedDynIterator::new(
                            object.map(DynResourceSeeAlso::<'a, C>::new),
                        )
                    },
                    is_defined_by: |ptr, context| unsafe {
                        let subject = &*(ptr as *const T);
                        let object = context.get(|context| subject.is_defined_by(context));
                        ::treeldr_rust_prelude::BoxedDynIterator::new(
                            object.map(DynResourceIsDefinedBy::<'a, C>::new),
                        )
                    },
                }
            }
        }
        pub struct DynResourceType<'d, C: ?Sized> {
            _p: ::std::marker::PhantomData<&'d C>,
            ptr: *const u8,
            tables: (ResourceDynTableInstance<C>, ClassDynTableInstance<C>),
        }
        impl<'d, C: ?Sized> DynResourceType<'d, C> {
            pub fn new<T: super::Class<C> + ::treeldr_rust_prelude::Reference<'d>>(
                value: T,
            ) -> Self {
                let ptr;
                let tables = (
                    {
                        let (p , t) = :: treeldr_rust_prelude :: AsTraitObject :: < ResourceDynTable :: < C > > :: into_trait_object (value) ;
                        ptr = p;
                        t
                    },
                    ::treeldr_rust_prelude::AsTraitObject::<ClassDynTable<C>>::into_trait_object(
                        value,
                    )
                    .1,
                );
                Self {
                    _p: ::std::marker::PhantomData,
                    ptr,
                    tables,
                }
            }
        }
        impl<'d, C: ?Sized> Clone for DynResourceType<'d, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'d, C: ?Sized> Copy for DynResourceType<'d, C> {}
        impl<'d, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for DynResourceType<'d, C> {}
        impl<'d, C: ?Sized> super::Resource<C> for DynResourceType<'d, C> {
            type Type < 'a > = DynResourceType :: < 'a , C > where Self : 'a , C : 'a ;
            type Types < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceType :: < 'a , C > > where Self : 'a , C : 'a ;
            type Label < 'a > = DynResourceLabel :: < 'a , C > where Self : 'a , C : 'a ;
            type Labels < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceLabel :: < 'a , C > > where Self : 'a , C : 'a ;
            type Comment < 'a > = DynResourceComment :: < 'a , C > where Self : 'a , C : 'a ;
            type Comments < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceComment :: < 'a , C > > where Self : 'a , C : 'a ;
            type SeeAlso < 'a > = DynResourceSeeAlso :: < 'a , C > where Self : 'a , C : 'a ;
            type SeeAlsos < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceSeeAlso :: < 'a , C > > where Self : 'a , C : 'a ;
            type IsDefinedBy < 'a > = DynResourceIsDefinedBy :: < 'a , C > where Self : 'a , C : 'a ;
            type IsDefinedBys < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceIsDefinedBy :: < 'a , C > > where Self : 'a , C : 'a ;
            fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
                unsafe {
                    (self.tables.0.type_)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
                unsafe {
                    (self.tables.0.label)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
                unsafe {
                    (self.tables.0.comment)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
                unsafe {
                    (self.tables.0.see_also)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
                unsafe {
                    (self.tables.0.is_defined_by)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
        }
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<ResourceDynTable<C>>
            for DynResourceType<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, ResourceDynTableInstance<C>) {
                (self.ptr, self.tables.0)
            }
            fn into_trait_object<'r>(self) -> (*const u8, ResourceDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.0)
            }
        }
        impl<'d, C: ?Sized> super::Class<C> for DynResourceType<'d, C> {
            type SubClassOf < 'a > = DynClassSubClassOf :: < 'a , C > where Self : 'a , C : 'a ;
            type SubClassOfs < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynClassSubClassOf :: < 'a , C > > where Self : 'a , C : 'a ;
            fn sub_class_of<'a>(&'a self, context: &'a C) -> Self::SubClassOfs<'a> {
                unsafe {
                    (self.tables.1.sub_class_of)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
        }
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<ClassDynTable<C>>
            for DynResourceType<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, ClassDynTableInstance<C>) {
                (self.ptr, self.tables.1)
            }
            fn into_trait_object<'r>(self) -> (*const u8, ClassDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.1)
            }
        }
        pub struct DynResourceLabel<'d, C: ?Sized> {
            _p: ::std::marker::PhantomData<&'d C>,
            ptr: *const u8,
            tables: (ResourceDynTableInstance<C>, LiteralDynTableInstance<C>),
        }
        impl<'d, C: ?Sized> DynResourceLabel<'d, C> {
            pub fn new<T: super::Literal<C> + ::treeldr_rust_prelude::Reference<'d>>(
                value: T,
            ) -> Self {
                let ptr;
                let tables = (
                    {
                        let (p , t) = :: treeldr_rust_prelude :: AsTraitObject :: < ResourceDynTable :: < C > > :: into_trait_object (value) ;
                        ptr = p;
                        t
                    },
                    ::treeldr_rust_prelude::AsTraitObject::<LiteralDynTable<C>>::into_trait_object(
                        value,
                    )
                    .1,
                );
                Self {
                    _p: ::std::marker::PhantomData,
                    ptr,
                    tables,
                }
            }
        }
        impl<'d, C: ?Sized> Clone for DynResourceLabel<'d, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'d, C: ?Sized> Copy for DynResourceLabel<'d, C> {}
        impl<'d, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for DynResourceLabel<'d, C> {}
        impl<'d, C: ?Sized> super::Resource<C> for DynResourceLabel<'d, C> {
            type Type < 'a > = DynResourceType :: < 'a , C > where Self : 'a , C : 'a ;
            type Types < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceType :: < 'a , C > > where Self : 'a , C : 'a ;
            type Label < 'a > = DynResourceLabel :: < 'a , C > where Self : 'a , C : 'a ;
            type Labels < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceLabel :: < 'a , C > > where Self : 'a , C : 'a ;
            type Comment < 'a > = DynResourceComment :: < 'a , C > where Self : 'a , C : 'a ;
            type Comments < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceComment :: < 'a , C > > where Self : 'a , C : 'a ;
            type SeeAlso < 'a > = DynResourceSeeAlso :: < 'a , C > where Self : 'a , C : 'a ;
            type SeeAlsos < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceSeeAlso :: < 'a , C > > where Self : 'a , C : 'a ;
            type IsDefinedBy < 'a > = DynResourceIsDefinedBy :: < 'a , C > where Self : 'a , C : 'a ;
            type IsDefinedBys < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceIsDefinedBy :: < 'a , C > > where Self : 'a , C : 'a ;
            fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
                unsafe {
                    (self.tables.0.type_)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
                unsafe {
                    (self.tables.0.label)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
                unsafe {
                    (self.tables.0.comment)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
                unsafe {
                    (self.tables.0.see_also)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
                unsafe {
                    (self.tables.0.is_defined_by)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
        }
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<ResourceDynTable<C>>
            for DynResourceLabel<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, ResourceDynTableInstance<C>) {
                (self.ptr, self.tables.0)
            }
            fn into_trait_object<'r>(self) -> (*const u8, ResourceDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.0)
            }
        }
        impl<'d, C: ?Sized> super::Literal<C> for DynResourceLabel<'d, C> {}
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<LiteralDynTable<C>>
            for DynResourceLabel<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, LiteralDynTableInstance<C>) {
                (self.ptr, self.tables.1)
            }
            fn into_trait_object<'r>(self) -> (*const u8, LiteralDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.1)
            }
        }
        pub struct DynResourceComment<'d, C: ?Sized> {
            _p: ::std::marker::PhantomData<&'d C>,
            ptr: *const u8,
            tables: (ResourceDynTableInstance<C>, LiteralDynTableInstance<C>),
        }
        impl<'d, C: ?Sized> DynResourceComment<'d, C> {
            pub fn new<T: super::Literal<C> + ::treeldr_rust_prelude::Reference<'d>>(
                value: T,
            ) -> Self {
                let ptr;
                let tables = (
                    {
                        let (p , t) = :: treeldr_rust_prelude :: AsTraitObject :: < ResourceDynTable :: < C > > :: into_trait_object (value) ;
                        ptr = p;
                        t
                    },
                    ::treeldr_rust_prelude::AsTraitObject::<LiteralDynTable<C>>::into_trait_object(
                        value,
                    )
                    .1,
                );
                Self {
                    _p: ::std::marker::PhantomData,
                    ptr,
                    tables,
                }
            }
        }
        impl<'d, C: ?Sized> Clone for DynResourceComment<'d, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'d, C: ?Sized> Copy for DynResourceComment<'d, C> {}
        impl<'d, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for DynResourceComment<'d, C> {}
        impl<'d, C: ?Sized> super::Resource<C> for DynResourceComment<'d, C> {
            type Type < 'a > = DynResourceType :: < 'a , C > where Self : 'a , C : 'a ;
            type Types < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceType :: < 'a , C > > where Self : 'a , C : 'a ;
            type Label < 'a > = DynResourceLabel :: < 'a , C > where Self : 'a , C : 'a ;
            type Labels < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceLabel :: < 'a , C > > where Self : 'a , C : 'a ;
            type Comment < 'a > = DynResourceComment :: < 'a , C > where Self : 'a , C : 'a ;
            type Comments < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceComment :: < 'a , C > > where Self : 'a , C : 'a ;
            type SeeAlso < 'a > = DynResourceSeeAlso :: < 'a , C > where Self : 'a , C : 'a ;
            type SeeAlsos < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceSeeAlso :: < 'a , C > > where Self : 'a , C : 'a ;
            type IsDefinedBy < 'a > = DynResourceIsDefinedBy :: < 'a , C > where Self : 'a , C : 'a ;
            type IsDefinedBys < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceIsDefinedBy :: < 'a , C > > where Self : 'a , C : 'a ;
            fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
                unsafe {
                    (self.tables.0.type_)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
                unsafe {
                    (self.tables.0.label)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
                unsafe {
                    (self.tables.0.comment)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
                unsafe {
                    (self.tables.0.see_also)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
                unsafe {
                    (self.tables.0.is_defined_by)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
        }
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<ResourceDynTable<C>>
            for DynResourceComment<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, ResourceDynTableInstance<C>) {
                (self.ptr, self.tables.0)
            }
            fn into_trait_object<'r>(self) -> (*const u8, ResourceDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.0)
            }
        }
        impl<'d, C: ?Sized> super::Literal<C> for DynResourceComment<'d, C> {}
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<LiteralDynTable<C>>
            for DynResourceComment<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, LiteralDynTableInstance<C>) {
                (self.ptr, self.tables.1)
            }
            fn into_trait_object<'r>(self) -> (*const u8, LiteralDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.1)
            }
        }
        pub struct DynResourceSeeAlso<'d, C: ?Sized> {
            _p: ::std::marker::PhantomData<&'d C>,
            ptr: *const u8,
            tables: (ResourceDynTableInstance<C>,),
        }
        impl<'d, C: ?Sized> DynResourceSeeAlso<'d, C> {
            pub fn new<T: super::Resource<C> + ::treeldr_rust_prelude::Reference<'d>>(
                value: T,
            ) -> Self {
                let ptr;
                let tables = ({
                    let (p , t) = :: treeldr_rust_prelude :: AsTraitObject :: < ResourceDynTable :: < C > > :: into_trait_object (value) ;
                    ptr = p;
                    t
                },);
                Self {
                    _p: ::std::marker::PhantomData,
                    ptr,
                    tables,
                }
            }
        }
        impl<'d, C: ?Sized> Clone for DynResourceSeeAlso<'d, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'d, C: ?Sized> Copy for DynResourceSeeAlso<'d, C> {}
        impl<'d, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for DynResourceSeeAlso<'d, C> {}
        impl<'d, C: ?Sized> super::Resource<C> for DynResourceSeeAlso<'d, C> {
            type Type < 'a > = DynResourceType :: < 'a , C > where Self : 'a , C : 'a ;
            type Types < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceType :: < 'a , C > > where Self : 'a , C : 'a ;
            type Label < 'a > = DynResourceLabel :: < 'a , C > where Self : 'a , C : 'a ;
            type Labels < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceLabel :: < 'a , C > > where Self : 'a , C : 'a ;
            type Comment < 'a > = DynResourceComment :: < 'a , C > where Self : 'a , C : 'a ;
            type Comments < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceComment :: < 'a , C > > where Self : 'a , C : 'a ;
            type SeeAlso < 'a > = DynResourceSeeAlso :: < 'a , C > where Self : 'a , C : 'a ;
            type SeeAlsos < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceSeeAlso :: < 'a , C > > where Self : 'a , C : 'a ;
            type IsDefinedBy < 'a > = DynResourceIsDefinedBy :: < 'a , C > where Self : 'a , C : 'a ;
            type IsDefinedBys < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceIsDefinedBy :: < 'a , C > > where Self : 'a , C : 'a ;
            fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
                unsafe {
                    (self.tables.0.type_)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
                unsafe {
                    (self.tables.0.label)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
                unsafe {
                    (self.tables.0.comment)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
                unsafe {
                    (self.tables.0.see_also)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
                unsafe {
                    (self.tables.0.is_defined_by)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
        }
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<ResourceDynTable<C>>
            for DynResourceSeeAlso<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, ResourceDynTableInstance<C>) {
                (self.ptr, self.tables.0)
            }
            fn into_trait_object<'r>(self) -> (*const u8, ResourceDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.0)
            }
        }
        pub struct DynResourceIsDefinedBy<'d, C: ?Sized> {
            _p: ::std::marker::PhantomData<&'d C>,
            ptr: *const u8,
            tables: (ResourceDynTableInstance<C>,),
        }
        impl<'d, C: ?Sized> DynResourceIsDefinedBy<'d, C> {
            pub fn new<T: super::Resource<C> + ::treeldr_rust_prelude::Reference<'d>>(
                value: T,
            ) -> Self {
                let ptr;
                let tables = ({
                    let (p , t) = :: treeldr_rust_prelude :: AsTraitObject :: < ResourceDynTable :: < C > > :: into_trait_object (value) ;
                    ptr = p;
                    t
                },);
                Self {
                    _p: ::std::marker::PhantomData,
                    ptr,
                    tables,
                }
            }
        }
        impl<'d, C: ?Sized> Clone for DynResourceIsDefinedBy<'d, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'d, C: ?Sized> Copy for DynResourceIsDefinedBy<'d, C> {}
        impl<'d, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for DynResourceIsDefinedBy<'d, C> {}
        impl<'d, C: ?Sized> super::Resource<C> for DynResourceIsDefinedBy<'d, C> {
            type Type < 'a > = DynResourceType :: < 'a , C > where Self : 'a , C : 'a ;
            type Types < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceType :: < 'a , C > > where Self : 'a , C : 'a ;
            type Label < 'a > = DynResourceLabel :: < 'a , C > where Self : 'a , C : 'a ;
            type Labels < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceLabel :: < 'a , C > > where Self : 'a , C : 'a ;
            type Comment < 'a > = DynResourceComment :: < 'a , C > where Self : 'a , C : 'a ;
            type Comments < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceComment :: < 'a , C > > where Self : 'a , C : 'a ;
            type SeeAlso < 'a > = DynResourceSeeAlso :: < 'a , C > where Self : 'a , C : 'a ;
            type SeeAlsos < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceSeeAlso :: < 'a , C > > where Self : 'a , C : 'a ;
            type IsDefinedBy < 'a > = DynResourceIsDefinedBy :: < 'a , C > where Self : 'a , C : 'a ;
            type IsDefinedBys < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceIsDefinedBy :: < 'a , C > > where Self : 'a , C : 'a ;
            fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
                unsafe {
                    (self.tables.0.type_)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
                unsafe {
                    (self.tables.0.label)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
                unsafe {
                    (self.tables.0.comment)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
                unsafe {
                    (self.tables.0.see_also)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
                unsafe {
                    (self.tables.0.is_defined_by)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
        }
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<ResourceDynTable<C>>
            for DynResourceIsDefinedBy<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, ResourceDynTableInstance<C>) {
                (self.ptr, self.tables.0)
            }
            fn into_trait_object<'r>(self) -> (*const u8, ResourceDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.0)
            }
        }
        pub struct ClassDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for ClassDynTable<C> {
            type Instance < 'a > = ClassDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct ClassDynTableInstance<'a, C: ?Sized> {
            pub sub_class_of: unsafe fn(
                *const u8,
                context: ::treeldr_rust_prelude::ContravariantReference<'a, C>,
            ) -> ::treeldr_rust_prelude::BoxedDynIterator<
                DynClassSubClassOf<'a, C>,
            >,
        }
        impl<'a, C: ?Sized> Clone for ClassDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for ClassDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> ClassDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Class<C>>() -> Self {
                Self {
                    sub_class_of: |ptr, context| unsafe {
                        let subject = &*(ptr as *const T);
                        let object = context.get(|context| subject.sub_class_of(context));
                        ::treeldr_rust_prelude::BoxedDynIterator::new(
                            object.map(DynClassSubClassOf::<'a, C>::new),
                        )
                    },
                }
            }
        }
        pub struct DynClassSubClassOf<'d, C: ?Sized> {
            _p: ::std::marker::PhantomData<&'d C>,
            ptr: *const u8,
            tables: (ResourceDynTableInstance<C>, ClassDynTableInstance<C>),
        }
        impl<'d, C: ?Sized> DynClassSubClassOf<'d, C> {
            pub fn new<T: super::Class<C> + ::treeldr_rust_prelude::Reference<'d>>(
                value: T,
            ) -> Self {
                let ptr;
                let tables = (
                    {
                        let (p , t) = :: treeldr_rust_prelude :: AsTraitObject :: < ResourceDynTable :: < C > > :: into_trait_object (value) ;
                        ptr = p;
                        t
                    },
                    ::treeldr_rust_prelude::AsTraitObject::<ClassDynTable<C>>::into_trait_object(
                        value,
                    )
                    .1,
                );
                Self {
                    _p: ::std::marker::PhantomData,
                    ptr,
                    tables,
                }
            }
        }
        impl<'d, C: ?Sized> Clone for DynClassSubClassOf<'d, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'d, C: ?Sized> Copy for DynClassSubClassOf<'d, C> {}
        impl<'d, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for DynClassSubClassOf<'d, C> {}
        impl<'d, C: ?Sized> super::Resource<C> for DynClassSubClassOf<'d, C> {
            type Type < 'a > = DynResourceType :: < 'a , C > where Self : 'a , C : 'a ;
            type Types < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceType :: < 'a , C > > where Self : 'a , C : 'a ;
            type Label < 'a > = DynResourceLabel :: < 'a , C > where Self : 'a , C : 'a ;
            type Labels < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceLabel :: < 'a , C > > where Self : 'a , C : 'a ;
            type Comment < 'a > = DynResourceComment :: < 'a , C > where Self : 'a , C : 'a ;
            type Comments < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceComment :: < 'a , C > > where Self : 'a , C : 'a ;
            type SeeAlso < 'a > = DynResourceSeeAlso :: < 'a , C > where Self : 'a , C : 'a ;
            type SeeAlsos < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceSeeAlso :: < 'a , C > > where Self : 'a , C : 'a ;
            type IsDefinedBy < 'a > = DynResourceIsDefinedBy :: < 'a , C > where Self : 'a , C : 'a ;
            type IsDefinedBys < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynResourceIsDefinedBy :: < 'a , C > > where Self : 'a , C : 'a ;
            fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
                unsafe {
                    (self.tables.0.type_)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
                unsafe {
                    (self.tables.0.label)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
                unsafe {
                    (self.tables.0.comment)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
                unsafe {
                    (self.tables.0.see_also)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
            fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
                unsafe {
                    (self.tables.0.is_defined_by)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
        }
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<ResourceDynTable<C>>
            for DynClassSubClassOf<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, ResourceDynTableInstance<C>) {
                (self.ptr, self.tables.0)
            }
            fn into_trait_object<'r>(self) -> (*const u8, ResourceDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.0)
            }
        }
        impl<'d, C: ?Sized> super::Class<C> for DynClassSubClassOf<'d, C> {
            type SubClassOf < 'a > = DynClassSubClassOf :: < 'a , C > where Self : 'a , C : 'a ;
            type SubClassOfs < 'a > = :: treeldr_rust_prelude :: BoxedDynIterator < 'a , DynClassSubClassOf :: < 'a , C > > where Self : 'a , C : 'a ;
            fn sub_class_of<'a>(&'a self, context: &'a C) -> Self::SubClassOfs<'a> {
                unsafe {
                    (self.tables.1.sub_class_of)(
                        self.ptr,
                        ::treeldr_rust_prelude::ContravariantReference::new(context),
                    )
                }
            }
        }
        unsafe impl<'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<ClassDynTable<C>>
            for DynClassSubClassOf<'d, C>
        {
            fn as_trait_object(&self) -> (*const u8, ClassDynTableInstance<C>) {
                (self.ptr, self.tables.1)
            }
            fn into_trait_object<'r>(self) -> (*const u8, ClassDynTableInstance<C>)
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.1)
            }
        }
        pub struct LiteralDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for LiteralDynTable<C> {
            type Instance < 'a > = LiteralDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct LiteralDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for LiteralDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for LiteralDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> LiteralDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Literal<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct DatatypeDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for DatatypeDynTable<C> {
            type Instance < 'a > = DatatypeDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct DatatypeDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for DatatypeDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for DatatypeDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> DatatypeDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Datatype<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
    }
    pub mod layout {
        #[derive(Clone, PartialEq, Eq, Ord, Debug, Default)]
        pub struct Resource<I> {
            id: Option<::treeldr_rust_prelude::Id<I>>,
        }
        impl<I> Resource<I> {
            fn new() -> Self {
                Self::default()
            }
        }
        impl<I, C: ?Sized + super::provider::ResourceProvider<I>> super::Resource<C> for Resource<I> {
            type Type < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type Types < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type Label < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type Labels < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type Comment < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type Comments < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type SeeAlso < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type SeeAlsos < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type IsDefinedBy < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type IsDefinedBys < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
                ::std::iter::empty()
            }
            fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
                ::std::iter::empty()
            }
            fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
                ::std::iter::empty()
            }
            fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
                ::std::iter::empty()
            }
            fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
                ::std::iter::empty()
            }
        }
        unsafe impl<I, C: ?Sized + super::provider::ResourceProvider<I>>
            ::treeldr_rust_prelude::AsTraitObject<super::trait_object::ResourceDynTable<C>>
            for Resource<I>
        {
            fn as_trait_object(
                &self,
            ) -> (*const u8, super::trait_object::ResourceDynTableInstance<C>) {
                let table = super::trait_object::ResourceDynTableInstance::<C>::new::<Self>();
                (self as *const Self as *const u8, table)
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace>
            ::treeldr_rust_prelude::IntoJsonLdSyntax<N> for Resource<N::Id>
        where
            N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
        {
            fn into_json_ld_syntax(
                self,
                namespace: &N,
            ) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
                let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
                if let Some(value) = self.id {
                    result.insert(
                        ::treeldr_rust_prelude::locspan::Meta("id".into(), ()),
                        ::treeldr_rust_prelude::locspan::Meta(
                            ::treeldr_rust_prelude::IntoJsonLdSyntax::into_json_ld_syntax(
                                value, namespace,
                            ),
                            (),
                        ),
                    );
                }
                result.into()
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::VocabularyMut>
            ::treeldr_rust_prelude::IntoJsonLdObjectMeta<N> for Resource<N::Id>
        where
            N: treeldr_rust_prelude::rdf_types::Namespace,
            N::Id: ::treeldr_rust_prelude::rdf_types::IntoId<Iri = N::Iri, BlankId = N::BlankId>,
            N::Iri: ::core::cmp::Eq + ::std::hash::Hash,
            N::BlankId: ::core::cmp::Eq + ::std::hash::Hash,
        {
            fn into_json_ld_object_meta(
                self,
                vocabulary: &mut N,
                meta: (),
            ) -> ::treeldr_rust_prelude::json_ld::IndexedObject<N::Iri, N::BlankId, ()>
            {
                let mut result = ::treeldr_rust_prelude::json_ld::Node::new();
                if let Some(value) = self.id {
                    result.properties_mut().insert(
                        ::treeldr_rust_prelude::locspan::Meta(
                            ::treeldr_rust_prelude::json_ld::Id::Valid(
                                ::treeldr_rust_prelude::json_ld::ValidId::Iri(
                                    vocabulary.insert(
                                        ::treeldr_rust_prelude::iref::Iri::new(
                                            "https://treeldr.org/self",
                                        )
                                        .unwrap(),
                                    ),
                                ),
                            ),
                            (),
                        ),
                        ::treeldr_rust_prelude::IntoJsonLdObjectMeta::into_json_ld_object_meta(
                            value, vocabulary, meta,
                        ),
                    );
                }
                ::treeldr_rust_prelude::locspan::Meta(
                    ::treeldr_rust_prelude::json_ld::Indexed::new(
                        ::treeldr_rust_prelude::json_ld::Object::Node(Box::new(result)),
                        None,
                    ),
                    meta,
                )
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
            ::treeldr_rust_prelude::FromRdf<N, V> for Resource<N::Id>
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            V: ::treeldr_rust_prelude::rdf::TypeCheck<N::Id>,
        {
            fn from_rdf<G>(
                namespace: &mut N,
                value: &::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
                graph: &G,
            ) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
            where
                G: ::treeldr_rust_prelude::grdf::Graph<
                    Subject = N::Id,
                    Predicate = N::Id,
                    Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
                >,
            {
                match value {
                    ::treeldr_rust_prelude::rdf_types::Object::Id(id) => Ok(Self {
                        id: { Some(::treeldr_rust_prelude::Id(id.clone())) },
                    }),
                    ::treeldr_rust_prelude::rdf_types::Object::Literal(literal) => {
                        Err(::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue)
                    }
                }
            }
        }
        pub struct ResourceQuadsAndValues<'a, I, V> {
            id_: Option<I>,
            _v: ::std::marker::PhantomData<&'a V>,
        }
        impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
            ::treeldr_rust_prelude::RdfIterator<N> for ResourceQuadsAndValues<'a, N::Id, V>
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
        {
            type Item = ::treeldr_rust_prelude::rdf::QuadOrValue<N::Id, V>;
            fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
                &mut self,
                vocabulary: &mut N,
                generator: &mut G,
                graph: Option<&N::Id>,
            ) -> Option<Self::Item> {
                self.id_
                    .take()
                    .map(::treeldr_rust_prelude::rdf_types::Object::Id)
                    .map(::treeldr_rust_prelude::rdf::QuadOrValue::Value)
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
            ::treeldr_rust_prelude::rdf::QuadsAndValues<N, V> for Resource<N::Id>
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
        {
            type QuadsAndValues < 'a > = ResourceQuadsAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
            fn unbound_rdf_quads_and_values<
                'a,
                G: ::treeldr_rust_prelude::rdf_types::Generator<N>,
            >(
                &'a self,
                namespace: &mut N,
                generator: &mut G,
            ) -> Self::QuadsAndValues<'a>
            where
                N::Id: 'a,
                V: 'a,
            {
                ResourceQuadsAndValues {
                    id_: Some(
                        self.id
                            .clone()
                            .map(::treeldr_rust_prelude::Id::unwrap)
                            .unwrap_or_else(|| generator.next(namespace)),
                    ),
                    _v: ::std::marker::PhantomData,
                }
            }
        }
    }
    pub mod provider {
        pub trait DatatypeProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Datatype>
        {
            type Datatype: super::Datatype<C>;
            fn get(&self, id: &I) -> Option<&Self::Datatype> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Datatype>>::get(self, id)
            }
        }
        pub trait LiteralProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Literal>
        {
            type Literal: super::Literal<C>;
            fn get(&self, id: &I) -> Option<&Self::Literal> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Literal>>::get(self, id)
            }
        }
        pub trait ResourceProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Resource>
        {
            type Resource: super::Resource<C>;
            fn get(&self, id: &I) -> Option<&Self::Resource> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Resource>>::get(self, id)
            }
        }
        pub trait ClassProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Class>
        {
            type Class: super::Class<C>;
            fn get(&self, id: &I) -> Option<&Self::Class> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Class>>::get(self, id)
            }
        }
    }
    pub trait Literal<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::LiteralDynTable<C>> + Resource<C>
    {
    }
    impl<C: ?Sized> Literal<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Literal<C>> Literal<C> for &'r T {}
    pub trait Resource<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::ResourceDynTable<C>>
    {
        type Type<'a>: ::treeldr_rust_prelude::Reference<'a> + Class<C>
        where
            Self: 'a,
            C: 'a;
        type Types<'a>: 'a + Iterator<Item = Self::Type<'a>>
        where
            Self: 'a,
            C: 'a;
        type Label<'a>: ::treeldr_rust_prelude::Reference<'a> + Literal<C>
        where
            Self: 'a,
            C: 'a;
        type Labels<'a>: 'a + Iterator<Item = Self::Label<'a>>
        where
            Self: 'a,
            C: 'a;
        type Comment<'a>: ::treeldr_rust_prelude::Reference<'a> + Literal<C>
        where
            Self: 'a,
            C: 'a;
        type Comments<'a>: 'a + Iterator<Item = Self::Comment<'a>>
        where
            Self: 'a,
            C: 'a;
        type SeeAlso<'a>: ::treeldr_rust_prelude::Reference<'a> + Resource<C>
        where
            Self: 'a,
            C: 'a;
        type SeeAlsos<'a>: 'a + Iterator<Item = Self::SeeAlso<'a>>
        where
            Self: 'a,
            C: 'a;
        type IsDefinedBy<'a>: ::treeldr_rust_prelude::Reference<'a> + Resource<C>
        where
            Self: 'a,
            C: 'a;
        type IsDefinedBys<'a>: 'a + Iterator<Item = Self::IsDefinedBy<'a>>
        where
            Self: 'a,
            C: 'a;
        fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a>;
        fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a>;
        fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a>;
        fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a>;
        fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a>;
    }
    impl<C: ?Sized> Resource<C> for ::std::convert::Infallible {
        type Type < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
        type Types < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
        type Label < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
        type Labels < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
        type Comment < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
        type Comments < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
        type SeeAlso < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
        type SeeAlsos < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
        type IsDefinedBy < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
        type IsDefinedBys < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
        fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
            unreachable!()
        }
        fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
            unreachable!()
        }
        fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
            unreachable!()
        }
        fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
            unreachable!()
        }
        fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
            unreachable!()
        }
    }
    impl<'r, C: ?Sized, T: Resource<C>> Resource<C> for &'r T {
        type Type < 'a > = T :: Type < 'a > where Self : 'a , C : 'a ;
        type Types < 'a > = T :: Types < 'a > where Self : 'a , C : 'a ;
        type Label < 'a > = T :: Label < 'a > where Self : 'a , C : 'a ;
        type Labels < 'a > = T :: Labels < 'a > where Self : 'a , C : 'a ;
        type Comment < 'a > = T :: Comment < 'a > where Self : 'a , C : 'a ;
        type Comments < 'a > = T :: Comments < 'a > where Self : 'a , C : 'a ;
        type SeeAlso < 'a > = T :: SeeAlso < 'a > where Self : 'a , C : 'a ;
        type SeeAlsos < 'a > = T :: SeeAlsos < 'a > where Self : 'a , C : 'a ;
        type IsDefinedBy < 'a > = T :: IsDefinedBy < 'a > where Self : 'a , C : 'a ;
        type IsDefinedBys < 'a > = T :: IsDefinedBys < 'a > where Self : 'a , C : 'a ;
        fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
            T::type_(*self, context)
        }
        fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
            T::label(*self, context)
        }
        fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
            T::comment(*self, context)
        }
        fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
            T::see_also(*self, context)
        }
        fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
            T::is_defined_by(*self, context)
        }
    }
    pub trait Class<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::ClassDynTable<C>> + Resource<C>
    {
        type SubClassOf<'a>: ::treeldr_rust_prelude::Reference<'a> + Class<C>
        where
            Self: 'a,
            C: 'a;
        type SubClassOfs<'a>: 'a + Iterator<Item = Self::SubClassOf<'a>>
        where
            Self: 'a,
            C: 'a;
        fn sub_class_of<'a>(&'a self, context: &'a C) -> Self::SubClassOfs<'a>;
    }
    impl<C: ?Sized> Class<C> for ::std::convert::Infallible {
        type SubClassOf < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
        type SubClassOfs < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
        fn sub_class_of<'a>(&'a self, context: &'a C) -> Self::SubClassOfs<'a> {
            unreachable!()
        }
    }
    impl<'r, C: ?Sized, T: Class<C>> Class<C> for &'r T {
        type SubClassOf < 'a > = T :: SubClassOf < 'a > where Self : 'a , C : 'a ;
        type SubClassOfs < 'a > = T :: SubClassOfs < 'a > where Self : 'a , C : 'a ;
        fn sub_class_of<'a>(&'a self, context: &'a C) -> Self::SubClassOfs<'a> {
            T::sub_class_of(*self, context)
        }
    }
    pub trait Datatype<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DatatypeDynTable<C>> + Class<C>
    {
    }
    impl<C: ?Sized> Datatype<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Datatype<C>> Datatype<C> for &'r T {}
}
pub mod xsd {
    pub mod trait_object {
        pub struct UnsignedByteDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for UnsignedByteDynTable<C> {
            type Instance < 'a > = UnsignedByteDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct UnsignedByteDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for UnsignedByteDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for UnsignedByteDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> UnsignedByteDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::UnsignedByte<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct IntDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for IntDynTable<C> {
            type Instance < 'a > = IntDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct IntDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for IntDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for IntDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> IntDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Int<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct DateTimeStampDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for DateTimeStampDynTable<C> {
            type Instance < 'a > = DateTimeStampDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct DateTimeStampDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for DateTimeStampDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for DateTimeStampDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> DateTimeStampDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::DateTimeStamp<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct Base64binaryDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for Base64binaryDynTable<C> {
            type Instance < 'a > = Base64binaryDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct Base64binaryDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for Base64binaryDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for Base64binaryDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> Base64binaryDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Base64binary<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct TokenDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for TokenDynTable<C> {
            type Instance < 'a > = TokenDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct TokenDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for TokenDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for TokenDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> TokenDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Token<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct DateTimeDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for DateTimeDynTable<C> {
            type Instance < 'a > = DateTimeDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct DateTimeDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for DateTimeDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for DateTimeDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> DateTimeDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::DateTime<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct DoubleDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for DoubleDynTable<C> {
            type Instance < 'a > = DoubleDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct DoubleDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for DoubleDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for DoubleDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> DoubleDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Double<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct AnyuriDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for AnyuriDynTable<C> {
            type Instance < 'a > = AnyuriDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct AnyuriDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for AnyuriDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for AnyuriDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> AnyuriDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Anyuri<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct QnameDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for QnameDynTable<C> {
            type Instance < 'a > = QnameDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct QnameDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for QnameDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for QnameDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> QnameDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Qname<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct FloatDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for FloatDynTable<C> {
            type Instance < 'a > = FloatDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct FloatDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for FloatDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for FloatDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> FloatDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Float<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct UnsignedShortDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for UnsignedShortDynTable<C> {
            type Instance < 'a > = UnsignedShortDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct UnsignedShortDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for UnsignedShortDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for UnsignedShortDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> UnsignedShortDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::UnsignedShort<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct NotationDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for NotationDynTable<C> {
            type Instance < 'a > = NotationDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct NotationDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for NotationDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for NotationDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> NotationDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Notation<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct IdrefsDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for IdrefsDynTable<C> {
            type Instance < 'a > = IdrefsDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct IdrefsDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for IdrefsDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for IdrefsDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> IdrefsDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Idrefs<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct EntitiesDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for EntitiesDynTable<C> {
            type Instance < 'a > = EntitiesDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct EntitiesDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for EntitiesDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for EntitiesDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> EntitiesDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Entities<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct PositiveIntegerDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for PositiveIntegerDynTable<C> {
            type Instance < 'a > = PositiveIntegerDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct PositiveIntegerDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for PositiveIntegerDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for PositiveIntegerDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> PositiveIntegerDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::PositiveInteger<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct NmtokensDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for NmtokensDynTable<C> {
            type Instance < 'a > = NmtokensDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct NmtokensDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for NmtokensDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for NmtokensDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> NmtokensDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Nmtokens<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct NegativeIntegerDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for NegativeIntegerDynTable<C> {
            type Instance < 'a > = NegativeIntegerDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct NegativeIntegerDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for NegativeIntegerDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for NegativeIntegerDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> NegativeIntegerDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::NegativeInteger<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct GYearDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for GYearDynTable<C> {
            type Instance < 'a > = GYearDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct GYearDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for GYearDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for GYearDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> GYearDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::GYear<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct NmtokenDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for NmtokenDynTable<C> {
            type Instance < 'a > = NmtokenDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct NmtokenDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for NmtokenDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for NmtokenDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> NmtokenDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Nmtoken<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct StringDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for StringDynTable<C> {
            type Instance < 'a > = StringDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct StringDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for StringDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for StringDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> StringDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::String<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct GMonthDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for GMonthDynTable<C> {
            type Instance < 'a > = GMonthDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct GMonthDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for GMonthDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for GMonthDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> GMonthDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::GMonth<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct IntegerDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for IntegerDynTable<C> {
            type Instance < 'a > = IntegerDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct IntegerDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for IntegerDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for IntegerDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> IntegerDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Integer<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct LongDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for LongDynTable<C> {
            type Instance < 'a > = LongDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct LongDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for LongDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for LongDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> LongDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Long<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct NonNegativeIntegerDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for NonNegativeIntegerDynTable<C> {
            type Instance < 'a > = NonNegativeIntegerDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct NonNegativeIntegerDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for NonNegativeIntegerDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for NonNegativeIntegerDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> NonNegativeIntegerDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::NonNegativeInteger<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct NormalizedStringDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for NormalizedStringDynTable<C> {
            type Instance < 'a > = NormalizedStringDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct NormalizedStringDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for NormalizedStringDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for NormalizedStringDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> NormalizedStringDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::NormalizedString<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct GMonthDayDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for GMonthDayDynTable<C> {
            type Instance < 'a > = GMonthDayDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct GMonthDayDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for GMonthDayDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for GMonthDayDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> GMonthDayDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::GMonthDay<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct EntityDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for EntityDynTable<C> {
            type Instance < 'a > = EntityDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct EntityDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for EntityDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for EntityDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> EntityDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Entity<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct TimeDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for TimeDynTable<C> {
            type Instance < 'a > = TimeDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct TimeDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for TimeDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for TimeDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> TimeDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Time<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct UnsignedLongDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for UnsignedLongDynTable<C> {
            type Instance < 'a > = UnsignedLongDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct UnsignedLongDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for UnsignedLongDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for UnsignedLongDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> UnsignedLongDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::UnsignedLong<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct ShortDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for ShortDynTable<C> {
            type Instance < 'a > = ShortDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct ShortDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for ShortDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for ShortDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> ShortDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Short<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct YearMonthDurationDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for YearMonthDurationDynTable<C> {
            type Instance < 'a > = YearMonthDurationDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct YearMonthDurationDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for YearMonthDurationDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for YearMonthDurationDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> YearMonthDurationDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::YearMonthDuration<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct LanguageDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for LanguageDynTable<C> {
            type Instance < 'a > = LanguageDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct LanguageDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for LanguageDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for LanguageDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> LanguageDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Language<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct NonPositiveIntegerDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for NonPositiveIntegerDynTable<C> {
            type Instance < 'a > = NonPositiveIntegerDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct NonPositiveIntegerDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for NonPositiveIntegerDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for NonPositiveIntegerDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> NonPositiveIntegerDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::NonPositiveInteger<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct BooleanDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for BooleanDynTable<C> {
            type Instance < 'a > = BooleanDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct BooleanDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for BooleanDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for BooleanDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> BooleanDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Boolean<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct DurationDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for DurationDynTable<C> {
            type Instance < 'a > = DurationDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct DurationDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for DurationDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for DurationDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> DurationDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Duration<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct GDayDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for GDayDynTable<C> {
            type Instance < 'a > = GDayDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct GDayDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for GDayDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for GDayDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> GDayDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::GDay<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct HexBinaryDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for HexBinaryDynTable<C> {
            type Instance < 'a > = HexBinaryDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct HexBinaryDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for HexBinaryDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for HexBinaryDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> HexBinaryDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::HexBinary<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct DateDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for DateDynTable<C> {
            type Instance < 'a > = DateDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct DateDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for DateDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for DateDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> DateDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Date<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct ByteDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for ByteDynTable<C> {
            type Instance < 'a > = ByteDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct ByteDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for ByteDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for ByteDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> ByteDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Byte<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct UnsignedIntDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for UnsignedIntDynTable<C> {
            type Instance < 'a > = UnsignedIntDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct UnsignedIntDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for UnsignedIntDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for UnsignedIntDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> UnsignedIntDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::UnsignedInt<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct IdDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for IdDynTable<C> {
            type Instance < 'a > = IdDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct IdDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for IdDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for IdDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> IdDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Id<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct DayTimeDurationDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for DayTimeDurationDynTable<C> {
            type Instance < 'a > = DayTimeDurationDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct DayTimeDurationDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for DayTimeDurationDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for DayTimeDurationDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> DayTimeDurationDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::DayTimeDuration<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct IdrefDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for IdrefDynTable<C> {
            type Instance < 'a > = IdrefDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct IdrefDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for IdrefDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for IdrefDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> IdrefDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Idref<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct NameDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for NameDynTable<C> {
            type Instance < 'a > = NameDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct NameDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for NameDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for NameDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> NameDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Name<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct NcnameDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for NcnameDynTable<C> {
            type Instance < 'a > = NcnameDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct NcnameDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for NcnameDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for NcnameDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> NcnameDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Ncname<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
        pub struct DecimalDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for DecimalDynTable<C> {
            type Instance < 'a > = DecimalDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct DecimalDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for DecimalDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for DecimalDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> DecimalDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Decimal<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
    }
    pub mod layout {
        pub type NonNegativeInteger = ::treeldr_rust_prelude::ty::NonNegativeInteger;
        pub type Decimal = f64;
        pub type Anyuri = ::treeldr_rust_prelude::iref::IriBuf;
        pub type HexBinary = ::treeldr_rust_prelude::ty::HexBytesBuf;
        pub type Double = f64;
        pub type NegativeInteger = ::treeldr_rust_prelude::ty::NegativeInteger;
        pub type Int = i32;
        pub type Short = i16;
        pub type Base64binary = ::treeldr_rust_prelude::ty::Base64BytesBuf;
        pub type Time = ::treeldr_rust_prelude::chrono::NaiveTime;
        pub type UnsignedInt = u32;
        pub type UnsignedShort = u16;
        pub type Byte = i8;
        pub type UnsignedByte = u8;
        pub type PositiveInteger = ::treeldr_rust_prelude::ty::PositiveInteger;
        pub type Boolean = bool;
        pub type DateTime =
            ::treeldr_rust_prelude::chrono::DateTime<::treeldr_rust_prelude::chrono::Utc>;
        pub type String = ::std::string::String;
        pub type Date = ::treeldr_rust_prelude::chrono::NaiveDate;
        pub type Long = i64;
        pub type UnsignedLong = u64;
        pub type Float = f32;
        pub type Integer = ::treeldr_rust_prelude::ty::Integer;
        pub type NonPositiveInteger = ::treeldr_rust_prelude::ty::NonPositiveInteger;
    }
    pub mod provider {
        pub trait TimeProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Time> {
            type Time: super::Time<C>;
            fn get(&self, id: &I) -> Option<&Self::Time> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Time>>::get(self, id)
            }
        }
        pub trait ByteProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Byte> {
            type Byte: super::Byte<C>;
            fn get(&self, id: &I) -> Option<&Self::Byte> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Byte>>::get(self, id)
            }
        }
        pub trait DayTimeDurationProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::DayTimeDuration>
        {
            type DayTimeDuration: super::DayTimeDuration<C>;
            fn get(&self, id: &I) -> Option<&Self::DayTimeDuration> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::DayTimeDuration>>::get(self, id)
            }
        }
        pub trait GMonthDayProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::GMonthDay>
        {
            type GMonthDay: super::GMonthDay<C>;
            fn get(&self, id: &I) -> Option<&Self::GMonthDay> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::GMonthDay>>::get(self, id)
            }
        }
        pub trait DateTimeProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::DateTime>
        {
            type DateTime: super::DateTime<C>;
            fn get(&self, id: &I) -> Option<&Self::DateTime> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::DateTime>>::get(self, id)
            }
        }
        pub trait FloatProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Float>
        {
            type Float: super::Float<C>;
            fn get(&self, id: &I) -> Option<&Self::Float> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Float>>::get(self, id)
            }
        }
        pub trait NcnameProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Ncname>
        {
            type Ncname: super::Ncname<C>;
            fn get(&self, id: &I) -> Option<&Self::Ncname> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Ncname>>::get(self, id)
            }
        }
        pub trait HexBinaryProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::HexBinary>
        {
            type HexBinary: super::HexBinary<C>;
            fn get(&self, id: &I) -> Option<&Self::HexBinary> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::HexBinary>>::get(self, id)
            }
        }
        pub trait BooleanProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Boolean>
        {
            type Boolean: super::Boolean<C>;
            fn get(&self, id: &I) -> Option<&Self::Boolean> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Boolean>>::get(self, id)
            }
        }
        pub trait GMonthProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::GMonth>
        {
            type GMonth: super::GMonth<C>;
            fn get(&self, id: &I) -> Option<&Self::GMonth> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::GMonth>>::get(self, id)
            }
        }
        pub trait NormalizedStringProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::NormalizedString>
        {
            type NormalizedString: super::NormalizedString<C>;
            fn get(&self, id: &I) -> Option<&Self::NormalizedString> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::NormalizedString>>::get(self, id)
            }
        }
        pub trait TokenProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Token>
        {
            type Token: super::Token<C>;
            fn get(&self, id: &I) -> Option<&Self::Token> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Token>>::get(self, id)
            }
        }
        pub trait LanguageProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Language>
        {
            type Language: super::Language<C>;
            fn get(&self, id: &I) -> Option<&Self::Language> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Language>>::get(self, id)
            }
        }
        pub trait NameProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Name> {
            type Name: super::Name<C>;
            fn get(&self, id: &I) -> Option<&Self::Name> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Name>>::get(self, id)
            }
        }
        pub trait DecimalProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Decimal>
        {
            type Decimal: super::Decimal<C>;
            fn get(&self, id: &I) -> Option<&Self::Decimal> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Decimal>>::get(self, id)
            }
        }
        pub trait DateTimeStampProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::DateTimeStamp>
        {
            type DateTimeStamp: super::DateTimeStamp<C>;
            fn get(&self, id: &I) -> Option<&Self::DateTimeStamp> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::DateTimeStamp>>::get(self, id)
            }
        }
        pub trait AnyuriProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Anyuri>
        {
            type Anyuri: super::Anyuri<C>;
            fn get(&self, id: &I) -> Option<&Self::Anyuri> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Anyuri>>::get(self, id)
            }
        }
        pub trait NonPositiveIntegerProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::NonPositiveInteger>
        {
            type NonPositiveInteger: super::NonPositiveInteger<C>;
            fn get(&self, id: &I) -> Option<&Self::NonPositiveInteger> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::NonPositiveInteger>>::get(
                    self, id,
                )
            }
        }
        pub trait IdrefsProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Idrefs>
        {
            type Idrefs: super::Idrefs<C>;
            fn get(&self, id: &I) -> Option<&Self::Idrefs> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Idrefs>>::get(self, id)
            }
        }
        pub trait EntityProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Entity>
        {
            type Entity: super::Entity<C>;
            fn get(&self, id: &I) -> Option<&Self::Entity> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Entity>>::get(self, id)
            }
        }
        pub trait NegativeIntegerProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::NegativeInteger>
        {
            type NegativeInteger: super::NegativeInteger<C>;
            fn get(&self, id: &I) -> Option<&Self::NegativeInteger> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::NegativeInteger>>::get(self, id)
            }
        }
        pub trait GYearProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::GYear>
        {
            type GYear: super::GYear<C>;
            fn get(&self, id: &I) -> Option<&Self::GYear> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::GYear>>::get(self, id)
            }
        }
        pub trait UnsignedByteProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::UnsignedByte>
        {
            type UnsignedByte: super::UnsignedByte<C>;
            fn get(&self, id: &I) -> Option<&Self::UnsignedByte> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::UnsignedByte>>::get(self, id)
            }
        }
        pub trait ShortProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Short>
        {
            type Short: super::Short<C>;
            fn get(&self, id: &I) -> Option<&Self::Short> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Short>>::get(self, id)
            }
        }
        pub trait NmtokensProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Nmtokens>
        {
            type Nmtokens: super::Nmtokens<C>;
            fn get(&self, id: &I) -> Option<&Self::Nmtokens> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Nmtokens>>::get(self, id)
            }
        }
        pub trait QnameProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Qname>
        {
            type Qname: super::Qname<C>;
            fn get(&self, id: &I) -> Option<&Self::Qname> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Qname>>::get(self, id)
            }
        }
        pub trait DateProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Date> {
            type Date: super::Date<C>;
            fn get(&self, id: &I) -> Option<&Self::Date> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Date>>::get(self, id)
            }
        }
        pub trait GDayProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::GDay> {
            type GDay: super::GDay<C>;
            fn get(&self, id: &I) -> Option<&Self::GDay> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::GDay>>::get(self, id)
            }
        }
        pub trait Base64binaryProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Base64binary>
        {
            type Base64binary: super::Base64binary<C>;
            fn get(&self, id: &I) -> Option<&Self::Base64binary> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Base64binary>>::get(self, id)
            }
        }
        pub trait NotationProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Notation>
        {
            type Notation: super::Notation<C>;
            fn get(&self, id: &I) -> Option<&Self::Notation> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Notation>>::get(self, id)
            }
        }
        pub trait NmtokenProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Nmtoken>
        {
            type Nmtoken: super::Nmtoken<C>;
            fn get(&self, id: &I) -> Option<&Self::Nmtoken> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Nmtoken>>::get(self, id)
            }
        }
        pub trait IdrefProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Idref>
        {
            type Idref: super::Idref<C>;
            fn get(&self, id: &I) -> Option<&Self::Idref> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Idref>>::get(self, id)
            }
        }
        pub trait EntitiesProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Entities>
        {
            type Entities: super::Entities<C>;
            fn get(&self, id: &I) -> Option<&Self::Entities> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Entities>>::get(self, id)
            }
        }
        pub trait LongProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Long> {
            type Long: super::Long<C>;
            fn get(&self, id: &I) -> Option<&Self::Long> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Long>>::get(self, id)
            }
        }
        pub trait UnsignedLongProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::UnsignedLong>
        {
            type UnsignedLong: super::UnsignedLong<C>;
            fn get(&self, id: &I) -> Option<&Self::UnsignedLong> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::UnsignedLong>>::get(self, id)
            }
        }
        pub trait NonNegativeIntegerProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::NonNegativeInteger>
        {
            type NonNegativeInteger: super::NonNegativeInteger<C>;
            fn get(&self, id: &I) -> Option<&Self::NonNegativeInteger> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::NonNegativeInteger>>::get(
                    self, id,
                )
            }
        }
        pub trait UnsignedShortProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::UnsignedShort>
        {
            type UnsignedShort: super::UnsignedShort<C>;
            fn get(&self, id: &I) -> Option<&Self::UnsignedShort> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::UnsignedShort>>::get(self, id)
            }
        }
        pub trait PositiveIntegerProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::PositiveInteger>
        {
            type PositiveInteger: super::PositiveInteger<C>;
            fn get(&self, id: &I) -> Option<&Self::PositiveInteger> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::PositiveInteger>>::get(self, id)
            }
        }
        pub trait YearMonthDurationProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::YearMonthDuration>
        {
            type YearMonthDuration: super::YearMonthDuration<C>;
            fn get(&self, id: &I) -> Option<&Self::YearMonthDuration> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::YearMonthDuration>>::get(
                    self, id,
                )
            }
        }
        pub trait IdProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Id> {
            type Id: super::Id<C>;
            fn get(&self, id: &I) -> Option<&Self::Id> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Id>>::get(self, id)
            }
        }
        pub trait StringProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::String>
        {
            type String: super::String<C>;
            fn get(&self, id: &I) -> Option<&Self::String> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::String>>::get(self, id)
            }
        }
        pub trait IntegerProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Integer>
        {
            type Integer: super::Integer<C>;
            fn get(&self, id: &I) -> Option<&Self::Integer> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Integer>>::get(self, id)
            }
        }
        pub trait DoubleProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Double>
        {
            type Double: super::Double<C>;
            fn get(&self, id: &I) -> Option<&Self::Double> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Double>>::get(self, id)
            }
        }
        pub trait IntProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Int> {
            type Int: super::Int<C>;
            fn get(&self, id: &I) -> Option<&Self::Int> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Int>>::get(self, id)
            }
        }
        pub trait DurationProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::Duration>
        {
            type Duration: super::Duration<C>;
            fn get(&self, id: &I) -> Option<&Self::Duration> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Duration>>::get(self, id)
            }
        }
        pub trait UnsignedIntProvider<I: ?Sized>:
            ::treeldr_rust_prelude::Provider<I, Self::UnsignedInt>
        {
            type UnsignedInt: super::UnsignedInt<C>;
            fn get(&self, id: &I) -> Option<&Self::UnsignedInt> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::UnsignedInt>>::get(self, id)
            }
        }
    }
    pub trait Long<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::LongDynTable<C>>
    {
    }
    impl<C: ?Sized> Long<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Long<C>> Long<C> for &'r T {}
    pub trait GDay<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::GDayDynTable<C>>
    {
    }
    impl<C: ?Sized> GDay<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: GDay<C>> GDay<C> for &'r T {}
    pub trait UnsignedByte<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::UnsignedByteDynTable<C>>
    {
    }
    impl<C: ?Sized> UnsignedByte<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: UnsignedByte<C>> UnsignedByte<C> for &'r T {}
    pub trait Byte<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::ByteDynTable<C>>
    {
    }
    impl<C: ?Sized> Byte<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Byte<C>> Byte<C> for &'r T {}
    pub trait Int<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::IntDynTable<C>>
    {
    }
    impl<C: ?Sized> Int<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Int<C>> Int<C> for &'r T {}
    pub trait GMonthDay<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::GMonthDayDynTable<C>>
    {
    }
    impl<C: ?Sized> GMonthDay<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: GMonthDay<C>> GMonthDay<C> for &'r T {}
    pub trait Duration<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DurationDynTable<C>>
    {
    }
    impl<C: ?Sized> Duration<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Duration<C>> Duration<C> for &'r T {}
    pub trait Base64binary<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::Base64binaryDynTable<C>>
    {
    }
    impl<C: ?Sized> Base64binary<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Base64binary<C>> Base64binary<C> for &'r T {}
    pub trait Notation<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NotationDynTable<C>>
    {
    }
    impl<C: ?Sized> Notation<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Notation<C>> Notation<C> for &'r T {}
    pub trait Ncname<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NcnameDynTable<C>>
    {
    }
    impl<C: ?Sized> Ncname<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Ncname<C>> Ncname<C> for &'r T {}
    pub trait Double<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DoubleDynTable<C>>
    {
    }
    impl<C: ?Sized> Double<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Double<C>> Double<C> for &'r T {}
    pub trait Boolean<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::BooleanDynTable<C>>
    {
    }
    impl<C: ?Sized> Boolean<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Boolean<C>> Boolean<C> for &'r T {}
    pub trait NormalizedString<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NormalizedStringDynTable<C>>
    {
    }
    impl<C: ?Sized> NormalizedString<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: NormalizedString<C>> NormalizedString<C> for &'r T {}
    pub trait Idref<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::IdrefDynTable<C>>
    {
    }
    impl<C: ?Sized> Idref<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Idref<C>> Idref<C> for &'r T {}
    pub trait Idrefs<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::IdrefsDynTable<C>>
    {
    }
    impl<C: ?Sized> Idrefs<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Idrefs<C>> Idrefs<C> for &'r T {}
    pub trait Qname<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::QnameDynTable<C>>
    {
    }
    impl<C: ?Sized> Qname<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Qname<C>> Qname<C> for &'r T {}
    pub trait Id<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::IdDynTable<C>>
    {
    }
    impl<C: ?Sized> Id<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Id<C>> Id<C> for &'r T {}
    pub trait NonNegativeInteger<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NonNegativeIntegerDynTable<C>>
    {
    }
    impl<C: ?Sized> NonNegativeInteger<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: NonNegativeInteger<C>> NonNegativeInteger<C> for &'r T {}
    pub trait DateTimeStamp<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DateTimeStampDynTable<C>>
    {
    }
    impl<C: ?Sized> DateTimeStamp<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: DateTimeStamp<C>> DateTimeStamp<C> for &'r T {}
    pub trait HexBinary<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::HexBinaryDynTable<C>>
    {
    }
    impl<C: ?Sized> HexBinary<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: HexBinary<C>> HexBinary<C> for &'r T {}
    pub trait Anyuri<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::AnyuriDynTable<C>>
    {
    }
    impl<C: ?Sized> Anyuri<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Anyuri<C>> Anyuri<C> for &'r T {}
    pub trait Time<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::TimeDynTable<C>>
    {
    }
    impl<C: ?Sized> Time<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Time<C>> Time<C> for &'r T {}
    pub trait Name<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NameDynTable<C>>
    {
    }
    impl<C: ?Sized> Name<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Name<C>> Name<C> for &'r T {}
    pub trait UnsignedLong<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::UnsignedLongDynTable<C>>
    {
    }
    impl<C: ?Sized> UnsignedLong<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: UnsignedLong<C>> UnsignedLong<C> for &'r T {}
    pub trait UnsignedInt<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::UnsignedIntDynTable<C>>
    {
    }
    impl<C: ?Sized> UnsignedInt<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: UnsignedInt<C>> UnsignedInt<C> for &'r T {}
    pub trait DayTimeDuration<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DayTimeDurationDynTable<C>>
    {
    }
    impl<C: ?Sized> DayTimeDuration<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: DayTimeDuration<C>> DayTimeDuration<C> for &'r T {}
    pub trait Entity<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::EntityDynTable<C>>
    {
    }
    impl<C: ?Sized> Entity<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Entity<C>> Entity<C> for &'r T {}
    pub trait PositiveInteger<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::PositiveIntegerDynTable<C>>
    {
    }
    impl<C: ?Sized> PositiveInteger<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: PositiveInteger<C>> PositiveInteger<C> for &'r T {}
    pub trait DateTime<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DateTimeDynTable<C>>
    {
    }
    impl<C: ?Sized> DateTime<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: DateTime<C>> DateTime<C> for &'r T {}
    pub trait Nmtoken<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NmtokenDynTable<C>>
    {
    }
    impl<C: ?Sized> Nmtoken<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Nmtoken<C>> Nmtoken<C> for &'r T {}
    pub trait Short<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::ShortDynTable<C>>
    {
    }
    impl<C: ?Sized> Short<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Short<C>> Short<C> for &'r T {}
    pub trait String<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::StringDynTable<C>>
    {
    }
    impl<C: ?Sized> String<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: String<C>> String<C> for &'r T {}
    pub trait NegativeInteger<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NegativeIntegerDynTable<C>>
    {
    }
    impl<C: ?Sized> NegativeInteger<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: NegativeInteger<C>> NegativeInteger<C> for &'r T {}
    pub trait Nmtokens<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NmtokensDynTable<C>>
    {
    }
    impl<C: ?Sized> Nmtokens<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Nmtokens<C>> Nmtokens<C> for &'r T {}
    pub trait YearMonthDuration<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::YearMonthDurationDynTable<C>>
    {
    }
    impl<C: ?Sized> YearMonthDuration<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: YearMonthDuration<C>> YearMonthDuration<C> for &'r T {}
    pub trait Token<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::TokenDynTable<C>>
    {
    }
    impl<C: ?Sized> Token<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Token<C>> Token<C> for &'r T {}
    pub trait GMonth<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::GMonthDynTable<C>>
    {
    }
    impl<C: ?Sized> GMonth<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: GMonth<C>> GMonth<C> for &'r T {}
    pub trait Float<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::FloatDynTable<C>>
    {
    }
    impl<C: ?Sized> Float<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Float<C>> Float<C> for &'r T {}
    pub trait GYear<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::GYearDynTable<C>>
    {
    }
    impl<C: ?Sized> GYear<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: GYear<C>> GYear<C> for &'r T {}
    pub trait Language<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::LanguageDynTable<C>>
    {
    }
    impl<C: ?Sized> Language<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Language<C>> Language<C> for &'r T {}
    pub trait NonPositiveInteger<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NonPositiveIntegerDynTable<C>>
    {
    }
    impl<C: ?Sized> NonPositiveInteger<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: NonPositiveInteger<C>> NonPositiveInteger<C> for &'r T {}
    pub trait Integer<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::IntegerDynTable<C>>
    {
    }
    impl<C: ?Sized> Integer<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Integer<C>> Integer<C> for &'r T {}
    pub trait Decimal<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DecimalDynTable<C>>
    {
    }
    impl<C: ?Sized> Decimal<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Decimal<C>> Decimal<C> for &'r T {}
    pub trait Entities<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::EntitiesDynTable<C>>
    {
    }
    impl<C: ?Sized> Entities<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Entities<C>> Entities<C> for &'r T {}
    pub trait UnsignedShort<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::UnsignedShortDynTable<C>>
    {
    }
    impl<C: ?Sized> UnsignedShort<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: UnsignedShort<C>> UnsignedShort<C> for &'r T {}
    pub trait Date<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DateDynTable<C>>
    {
    }
    impl<C: ?Sized> Date<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Date<C>> Date<C> for &'r T {}
    impl<I, C: ?Sized> UnsignedByte<C> for u8 {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::UnsignedByteDynTable<C>> for u8
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::UnsignedByteDynTableInstance<C>) {
            let table = trait_object::UnsignedByteDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> NegativeInteger<C> for ::treeldr_rust_prelude::ty::NegativeInteger {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NegativeIntegerDynTable<C>>
        for ::treeldr_rust_prelude::ty::NegativeInteger
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::NegativeIntegerDynTableInstance<C>) {
            let table = trait_object::NegativeIntegerDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Short<C> for i16 {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::ShortDynTable<C>>
        for i16
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::ShortDynTableInstance<C>) {
            let table = trait_object::ShortDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Boolean<C> for bool {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::BooleanDynTable<C>> for bool
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::BooleanDynTableInstance<C>) {
            let table = trait_object::BooleanDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Byte<C> for i8 {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::ByteDynTable<C>>
        for i8
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::ByteDynTableInstance<C>) {
            let table = trait_object::ByteDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Date<C> for ::treeldr_rust_prelude::chrono::NaiveDate {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::DateDynTable<C>>
        for ::treeldr_rust_prelude::chrono::NaiveDate
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::DateDynTableInstance<C>) {
            let table = trait_object::DateDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> NonPositiveInteger<C> for ::treeldr_rust_prelude::ty::NonPositiveInteger {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NonPositiveIntegerDynTable<C>>
        for ::treeldr_rust_prelude::ty::NonPositiveInteger
    {
        fn as_trait_object(
            &self,
        ) -> (
            *const u8,
            trait_object::NonPositiveIntegerDynTableInstance<C>,
        ) {
            let table = trait_object::NonPositiveIntegerDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> UnsignedShort<C> for u16 {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::UnsignedShortDynTable<C>> for u16
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::UnsignedShortDynTableInstance<C>) {
            let table = trait_object::UnsignedShortDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Integer<C> for ::treeldr_rust_prelude::ty::Integer {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::IntegerDynTable<C>>
        for ::treeldr_rust_prelude::ty::Integer
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::IntegerDynTableInstance<C>) {
            let table = trait_object::IntegerDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Double<C> for f64 {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::DoubleDynTable<C>>
        for f64
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::DoubleDynTableInstance<C>) {
            let table = trait_object::DoubleDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> String<C> for ::std::string::String {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::StringDynTable<C>>
        for ::std::string::String
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::StringDynTableInstance<C>) {
            let table = trait_object::StringDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Int<C> for i32 {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::IntDynTable<C>>
        for i32
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::IntDynTableInstance<C>) {
            let table = trait_object::IntDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Decimal<C> for f64 {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DecimalDynTable<C>> for f64
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::DecimalDynTableInstance<C>) {
            let table = trait_object::DecimalDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> NonNegativeInteger<C> for ::treeldr_rust_prelude::ty::NonNegativeInteger {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::NonNegativeIntegerDynTable<C>>
        for ::treeldr_rust_prelude::ty::NonNegativeInteger
    {
        fn as_trait_object(
            &self,
        ) -> (
            *const u8,
            trait_object::NonNegativeIntegerDynTableInstance<C>,
        ) {
            let table = trait_object::NonNegativeIntegerDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Anyuri<C> for ::treeldr_rust_prelude::iref::IriBuf {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::AnyuriDynTable<C>>
        for ::treeldr_rust_prelude::iref::IriBuf
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::AnyuriDynTableInstance<C>) {
            let table = trait_object::AnyuriDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Time<C> for ::treeldr_rust_prelude::chrono::NaiveTime {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::TimeDynTable<C>>
        for ::treeldr_rust_prelude::chrono::NaiveTime
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::TimeDynTableInstance<C>) {
            let table = trait_object::TimeDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> HexBinary<C> for ::treeldr_rust_prelude::ty::HexBytesBuf {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::HexBinaryDynTable<C>>
        for ::treeldr_rust_prelude::ty::HexBytesBuf
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::HexBinaryDynTableInstance<C>) {
            let table = trait_object::HexBinaryDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Base64binary<C> for ::treeldr_rust_prelude::ty::Base64BytesBuf {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::Base64binaryDynTable<C>>
        for ::treeldr_rust_prelude::ty::Base64BytesBuf
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::Base64binaryDynTableInstance<C>) {
            let table = trait_object::Base64binaryDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> UnsignedLong<C> for u64 {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::UnsignedLongDynTable<C>> for u64
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::UnsignedLongDynTableInstance<C>) {
            let table = trait_object::UnsignedLongDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Float<C> for f32 {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::FloatDynTable<C>>
        for f32
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::FloatDynTableInstance<C>) {
            let table = trait_object::FloatDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> Long<C> for i64 {}
    unsafe impl<I, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<trait_object::LongDynTable<C>>
        for i64
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::LongDynTableInstance<C>) {
            let table = trait_object::LongDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> DateTime<C>
        for ::treeldr_rust_prelude::chrono::DateTime<::treeldr_rust_prelude::chrono::Utc>
    {
    }
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::DateTimeDynTable<C>>
        for ::treeldr_rust_prelude::chrono::DateTime<::treeldr_rust_prelude::chrono::Utc>
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::DateTimeDynTableInstance<C>) {
            let table = trait_object::DateTimeDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> PositiveInteger<C> for ::treeldr_rust_prelude::ty::PositiveInteger {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::PositiveIntegerDynTable<C>>
        for ::treeldr_rust_prelude::ty::PositiveInteger
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::PositiveIntegerDynTableInstance<C>) {
            let table = trait_object::PositiveIntegerDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
    impl<I, C: ?Sized> UnsignedInt<C> for u32 {}
    unsafe impl<I, C: ?Sized>
        ::treeldr_rust_prelude::AsTraitObject<trait_object::UnsignedIntDynTable<C>> for u32
    {
        fn as_trait_object(&self) -> (*const u8, trait_object::UnsignedIntDynTableInstance<C>) {
            let table = trait_object::UnsignedIntDynTableInstance::<C>::new::<Self>();
            (self as *const Self as *const u8, table)
        }
    }
}
pub mod example {
    pub mod layout {
        #[derive(Clone, PartialEq, Eq, Ord, Debug)]
        pub enum Enum {
            A(A),
            B(B),
        }
        #[derive(Clone, PartialEq, Eq, Ord, Debug, Default)]
        pub struct A {
            a: Option<super::super::xsd::layout::String>,
        }
        impl A {
            fn new() -> Self {
                Self::default()
            }
        }
        #[derive(Clone, PartialEq, Eq, Ord, Debug, Default)]
        pub struct B {
            b: Option<super::super::xsd::layout::String>,
        }
        impl B {
            fn new() -> Self {
                Self::default()
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace>
            ::treeldr_rust_prelude::IntoJsonLdSyntax<N> for B
        where
            N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
        {
            fn into_json_ld_syntax(
                self,
                namespace: &N,
            ) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
                let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
                if let Some(value) = self.b {
                    result.insert(
                        ::treeldr_rust_prelude::locspan::Meta("b".into(), ()),
                        ::treeldr_rust_prelude::locspan::Meta(
                            ::treeldr_rust_prelude::IntoJsonLdSyntax::into_json_ld_syntax(
                                value, namespace,
                            ),
                            (),
                        ),
                    );
                }
                result.into()
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::VocabularyMut>
            ::treeldr_rust_prelude::IntoJsonLdObjectMeta<N> for Enum
        where
            N: treeldr_rust_prelude::rdf_types::Namespace,
            N::Id: ::treeldr_rust_prelude::rdf_types::IntoId<Iri = N::Iri, BlankId = N::BlankId>,
            N::Iri: ::core::cmp::Eq + ::std::hash::Hash,
            N::BlankId: ::core::cmp::Eq + ::std::hash::Hash,
        {
            fn into_json_ld_object_meta(
                self,
                vocabulary: &mut N,
                meta: (),
            ) -> ::treeldr_rust_prelude::json_ld::IndexedObject<N::Iri, N::BlankId, ()>
            {
                match self {
                    Self::A(value) => value.into_json_ld_object_meta(vocabulary, meta),
                    Self::B(value) => value.into_json_ld_object_meta(vocabulary, meta),
                }
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace>
            ::treeldr_rust_prelude::IntoJsonLdSyntax<N> for A
        where
            N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
        {
            fn into_json_ld_syntax(
                self,
                namespace: &N,
            ) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
                let mut result = ::treeldr_rust_prelude::json_ld::syntax::Object::new();
                if let Some(value) = self.a {
                    result.insert(
                        ::treeldr_rust_prelude::locspan::Meta("a".into(), ()),
                        ::treeldr_rust_prelude::locspan::Meta(
                            ::treeldr_rust_prelude::IntoJsonLdSyntax::into_json_ld_syntax(
                                value, namespace,
                            ),
                            (),
                        ),
                    );
                }
                result.into()
            }
        }
        impl<I, C: ?Sized> super::A<C> for A {
            type A < 'a > = & 'a example :: xsd :: layout :: String where Self : 'a , C : 'a ;
            fn a<'a>(&'a self, context: &'a C) -> Option<Self::A<'a>> {
                self.a.as_ref()
            }
        }
        unsafe impl<I, C: ?Sized>
            ::treeldr_rust_prelude::AsTraitObject<super::trait_object::ADynTable<C>> for A
        {
            fn as_trait_object(&self) -> (*const u8, super::trait_object::ADynTableInstance<C>) {
                let table = super::trait_object::ADynTableInstance::<C>::new::<Self>();
                (self as *const Self as *const u8, table)
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace>
            ::treeldr_rust_prelude::IntoJsonLdSyntax<N> for Enum
        where
            N::Id: ::treeldr_rust_prelude::contextual::DisplayWithContext<N>,
        {
            fn into_json_ld_syntax(
                self,
                namespace: &N,
            ) -> ::treeldr_rust_prelude::json_ld::syntax::Value {
                match self {
                    Self::A(value) => value.into_json_ld_syntax(namespace),
                    Self::B(value) => value.into_json_ld_syntax(namespace),
                }
            }
        }
        impl<I, C: ?Sized> super::B<C> for B {
            type B < 'a > = & 'a example :: xsd :: layout :: String where Self : 'a , C : 'a ;
            fn b<'a>(&'a self, context: &'a C) -> Option<Self::B<'a>> {
                self.b.as_ref()
            }
        }
        unsafe impl<I, C: ?Sized>
            ::treeldr_rust_prelude::AsTraitObject<super::trait_object::BDynTable<C>> for B
        {
            fn as_trait_object(&self) -> (*const u8, super::trait_object::BDynTableInstance<C>) {
                let table = super::trait_object::BDynTableInstance::<C>::new::<Self>();
                (self as *const Self as *const u8, table)
            }
        }
        impl<I, C: ?Sized> super::Enum<C> for Enum {}
        unsafe impl<I, C: ?Sized>
            ::treeldr_rust_prelude::AsTraitObject<super::trait_object::EnumDynTable<C>> for Enum
        {
            fn as_trait_object(&self) -> (*const u8, super::trait_object::EnumDynTableInstance<C>) {
                let table = super::trait_object::EnumDynTableInstance::<C>::new::<Self>();
                (self as *const Self as *const u8, table)
            }
        }
        pub struct AQuadsAndValues<'a, I, V> {
            id_: Option<I>,
            a: ::treeldr_rust_prelude::rdf::iter::Optional<
                ::treeldr_rust_prelude::rdf::ValuesOnly<
                    ::treeldr_rust_prelude::rdf::LiteralValue<'a, ::std::string::String, I, V>,
                >,
            >,
        }
        impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
            ::treeldr_rust_prelude::RdfIterator<N> for AQuadsAndValues<'a, N::Id, V>
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            ::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
        {
            type Item = ::treeldr_rust_prelude::rdf::QuadOrValue<N::Id, V>;
            fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
                &mut self,
                vocabulary: &mut N,
                generator: &mut G,
                graph: Option<&N::Id>,
            ) -> Option<Self::Item> {
                self.a
                    .next_with(vocabulary, generator, graph)
                    .map(|item| match item {
                        ::treeldr_rust_prelude::rdf::QuadOrValue::Quad(quad) => {
                            treeldr_rust_prelude::rdf::QuadOrValue::Quad(quad)
                        }
                        treeldr_rust_prelude::rdf::QuadOrValue::Value(value) => {
                            treeldr_rust_prelude::rdf::QuadOrValue::Quad(::rdf_types::Quad(
                                self.id_.clone().unwrap(),
                                treeldr_rust_prelude::rdf_types::FromIri::from_iri(
                                    vocabulary.insert(::treeldr_rust_prelude::static_iref::iri!(
                                        "https://example.com/a"
                                    )),
                                ),
                                value,
                                graph.cloned(),
                            ))
                        }
                    })
                    .or_else(|| {
                        self.id_
                            .take()
                            .map(::treeldr_rust_prelude::rdf_types::Object::Id)
                            .map(::treeldr_rust_prelude::rdf::QuadOrValue::Value)
                    })
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
            ::treeldr_rust_prelude::rdf::QuadsAndValues<N, V> for A
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            ::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
        {
            type QuadsAndValues < 'a > = AQuadsAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
            fn unbound_rdf_quads_and_values<
                'a,
                G: ::treeldr_rust_prelude::rdf_types::Generator<N>,
            >(
                &'a self,
                namespace: &mut N,
                generator: &mut G,
            ) -> Self::QuadsAndValues<'a>
            where
                N::Id: 'a,
                V: 'a,
            {
                AQuadsAndValues {
                    id_: Some(generator.next(namespace)),
                    a: self.a.unbound_rdf_quads_and_values(namespace, generator),
                }
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
            ::treeldr_rust_prelude::FromRdf<N, V> for B
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            V: ::treeldr_rust_prelude::rdf::TypeCheck<N::Id>,
            ::std::string::String: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
        {
            fn from_rdf<G>(
                namespace: &mut N,
                value: &::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
                graph: &G,
            ) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
            where
                G: ::treeldr_rust_prelude::grdf::Graph<
                    Subject = N::Id,
                    Predicate = N::Id,
                    Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
                >,
            {
                match value {
                    ::treeldr_rust_prelude::rdf_types::Object::Id(id) => Ok(Self {
                        b: {
                            let mut objects = graph.objects(
                                &id,
                                &::treeldr_rust_prelude::rdf_types::FromIri::from_iri(
                                    namespace.insert(::treeldr_rust_prelude::static_iref::iri!(
                                        "https://example.com/b"
                                    )),
                                ),
                            );
                            let object = objects.next();
                            if objects.next().is_some() {
                                panic!("multiples values on functional property")
                            }
                            match object {
                                Some(object) => Some({
                                    ::treeldr_rust_prelude::FromRdf::from_rdf(
                                        namespace, object, graph,
                                    )?
                                }),
                                None => None,
                            }
                        },
                    }),
                    ::treeldr_rust_prelude::rdf_types::Object::Literal(literal) => {
                        Err(::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue)
                    }
                }
            }
        }
        pub struct BQuadsAndValues<'a, I, V> {
            id_: Option<I>,
            b: ::treeldr_rust_prelude::rdf::iter::Optional<
                ::treeldr_rust_prelude::rdf::ValuesOnly<
                    ::treeldr_rust_prelude::rdf::LiteralValue<'a, ::std::string::String, I, V>,
                >,
            >,
        }
        impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
            ::treeldr_rust_prelude::RdfIterator<N> for BQuadsAndValues<'a, N::Id, V>
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            ::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
        {
            type Item = ::treeldr_rust_prelude::rdf::QuadOrValue<N::Id, V>;
            fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
                &mut self,
                vocabulary: &mut N,
                generator: &mut G,
                graph: Option<&N::Id>,
            ) -> Option<Self::Item> {
                self.b
                    .next_with(vocabulary, generator, graph)
                    .map(|item| match item {
                        ::treeldr_rust_prelude::rdf::QuadOrValue::Quad(quad) => {
                            treeldr_rust_prelude::rdf::QuadOrValue::Quad(quad)
                        }
                        treeldr_rust_prelude::rdf::QuadOrValue::Value(value) => {
                            treeldr_rust_prelude::rdf::QuadOrValue::Quad(::rdf_types::Quad(
                                self.id_.clone().unwrap(),
                                treeldr_rust_prelude::rdf_types::FromIri::from_iri(
                                    vocabulary.insert(::treeldr_rust_prelude::static_iref::iri!(
                                        "https://example.com/b"
                                    )),
                                ),
                                value,
                                graph.cloned(),
                            ))
                        }
                    })
                    .or_else(|| {
                        self.id_
                            .take()
                            .map(::treeldr_rust_prelude::rdf_types::Object::Id)
                            .map(::treeldr_rust_prelude::rdf::QuadOrValue::Value)
                    })
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
            ::treeldr_rust_prelude::rdf::QuadsAndValues<N, V> for B
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            ::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
        {
            type QuadsAndValues < 'a > = BQuadsAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
            fn unbound_rdf_quads_and_values<
                'a,
                G: ::treeldr_rust_prelude::rdf_types::Generator<N>,
            >(
                &'a self,
                namespace: &mut N,
                generator: &mut G,
            ) -> Self::QuadsAndValues<'a>
            where
                N::Id: 'a,
                V: 'a,
            {
                BQuadsAndValues {
                    id_: Some(generator.next(namespace)),
                    b: self.b.unbound_rdf_quads_and_values(namespace, generator),
                }
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
            ::treeldr_rust_prelude::FromRdf<N, V> for Enum
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            V: ::treeldr_rust_prelude::rdf::TypeCheck<N::Id>,
            ::std::string::String: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
        {
            fn from_rdf<G>(
                namespace: &mut N,
                value: &::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
                graph: &G,
            ) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
            where
                G: ::treeldr_rust_prelude::grdf::Graph<
                    Subject = N::Id,
                    Predicate = N::Id,
                    Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
                >,
            {
                match value {
                    ::treeldr_rust_prelude::rdf_types::Object::Id(id) => {
                        if graph
                            .any_match(::treeldr_rust_prelude::rdf_types::Triple(
                                Some(id),
                                Some(&::treeldr_rust_prelude::rdf_types::FromIri::from_iri(
                                    namespace.insert(::treeldr_rust_prelude::static_iref::iri!(
                                        "https://example.com/a"
                                    )),
                                )),
                                None,
                            ))
                            .is_some()
                        {
                            Ok(Self::A(::treeldr_rust_prelude::rdf::FromRdf::from_rdf(
                                namespace, value, graph,
                            )?))
                        } else {
                            Ok(Self::B(::treeldr_rust_prelude::rdf::FromRdf::from_rdf(
                                namespace, value, graph,
                            )?))
                        }
                    }
                    ::treeldr_rust_prelude::rdf_types::Object::Literal(literal) => {
                        Err(::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue)
                    }
                }
            }
        }
        pub enum EnumQuadsAndValues<'a, I, V> {
            A(AQuadsAndValues<'a, I, V>),
            B(BQuadsAndValues<'a, I, V>),
        }
        impl<'a, N: ::treeldr_rust_prelude::rdf_types::Namespace, V: 'a>
            ::treeldr_rust_prelude::RdfIterator<N> for EnumQuadsAndValues<'a, N::Id, V>
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: 'a + Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            ::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
        {
            type Item = ::treeldr_rust_prelude::rdf::QuadOrValue<N::Id, V>;
            fn next_with<G: ::treeldr_rust_prelude::rdf_types::Generator<N>>(
                &mut self,
                vocabulary: &mut N,
                generator: &mut G,
                graph: Option<&N::Id>,
            ) -> Option<Self::Item> {
                match self {
                    Self::A(inner) => inner.next_with(namespace, generator, graph),
                    Self::B(inner) => inner.next_with(namespace, generator, graph),
                }
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
            ::treeldr_rust_prelude::rdf::QuadsAndValues<N, V> for Enum
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: Clone + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            ::std::string::String: ::treeldr_rust_prelude::rdf::AsLiteral<N, V>,
        {
            type QuadsAndValues < 'a > = EnumQuadsAndValues < 'a , N :: Id , V > where Self : 'a , N :: Id : 'a , V : 'a ;
            fn unbound_rdf_quads_and_values<
                'a,
                G: ::treeldr_rust_prelude::rdf_types::Generator<N>,
            >(
                &'a self,
                namespace: &mut N,
                generator: &mut G,
            ) -> Self::QuadsAndValues<'a>
            where
                N::Id: 'a,
                V: 'a,
            {
                match self {
                    Self::A(value) => EnumQuadsAndValues::A(
                        value.unbound_rdf_quads_and_values(namespace, generator),
                    ),
                    Self::B(value) => EnumQuadsAndValues::B(
                        value.unbound_rdf_quads_and_values(namespace, generator),
                    ),
                }
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::VocabularyMut>
            ::treeldr_rust_prelude::IntoJsonLdObjectMeta<N> for B
        where
            N: treeldr_rust_prelude::rdf_types::Namespace,
            N::Id: ::treeldr_rust_prelude::rdf_types::IntoId<Iri = N::Iri, BlankId = N::BlankId>,
            N::Iri: ::core::cmp::Eq + ::std::hash::Hash,
            N::BlankId: ::core::cmp::Eq + ::std::hash::Hash,
        {
            fn into_json_ld_object_meta(
                self,
                vocabulary: &mut N,
                meta: (),
            ) -> ::treeldr_rust_prelude::json_ld::IndexedObject<N::Iri, N::BlankId, ()>
            {
                let mut result = ::treeldr_rust_prelude::json_ld::Node::new();
                if let Some(value) = self.b {
                    result.properties_mut().insert(
                        ::treeldr_rust_prelude::locspan::Meta(
                            ::treeldr_rust_prelude::json_ld::Id::Valid(
                                ::treeldr_rust_prelude::json_ld::ValidId::Iri(
                                    vocabulary.insert(
                                        ::treeldr_rust_prelude::iref::Iri::new(
                                            "https://example.com/b",
                                        )
                                        .unwrap(),
                                    ),
                                ),
                            ),
                            (),
                        ),
                        ::treeldr_rust_prelude::IntoJsonLdObjectMeta::into_json_ld_object_meta(
                            value, vocabulary, meta,
                        ),
                    );
                }
                ::treeldr_rust_prelude::locspan::Meta(
                    ::treeldr_rust_prelude::json_ld::Indexed::new(
                        ::treeldr_rust_prelude::json_ld::Object::Node(Box::new(result)),
                        None,
                    ),
                    meta,
                )
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V>
            ::treeldr_rust_prelude::FromRdf<N, V> for A
        where
            N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
            N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri = N::Iri>,
            V: ::treeldr_rust_prelude::rdf::TypeCheck<N::Id>,
            ::std::string::String: ::treeldr_rust_prelude::rdf::FromLiteral<V, N>,
        {
            fn from_rdf<G>(
                namespace: &mut N,
                value: &::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
                graph: &G,
            ) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
            where
                G: ::treeldr_rust_prelude::grdf::Graph<
                    Subject = N::Id,
                    Predicate = N::Id,
                    Object = ::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
                >,
            {
                match value {
                    ::treeldr_rust_prelude::rdf_types::Object::Id(id) => Ok(Self {
                        a: {
                            let mut objects = graph.objects(
                                &id,
                                &::treeldr_rust_prelude::rdf_types::FromIri::from_iri(
                                    namespace.insert(::treeldr_rust_prelude::static_iref::iri!(
                                        "https://example.com/a"
                                    )),
                                ),
                            );
                            let object = objects.next();
                            if objects.next().is_some() {
                                panic!("multiples values on functional property")
                            }
                            match object {
                                Some(object) => Some({
                                    ::treeldr_rust_prelude::FromRdf::from_rdf(
                                        namespace, object, graph,
                                    )?
                                }),
                                None => None,
                            }
                        },
                    }),
                    ::treeldr_rust_prelude::rdf_types::Object::Literal(literal) => {
                        Err(::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue)
                    }
                }
            }
        }
        impl<I, C: ?Sized> super::super::rdfs::Resource<C> for B {
            type Type < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type Types < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type Label < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type Labels < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type Comment < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type Comments < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type SeeAlso < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type SeeAlsos < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type IsDefinedBy < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type IsDefinedBys < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
                ::std::iter::empty()
            }
            fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
                ::std::iter::empty()
            }
            fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
                ::std::iter::empty()
            }
            fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
                ::std::iter::empty()
            }
            fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
                ::std::iter::empty()
            }
        }
        unsafe impl<I, C: ?Sized>
            ::treeldr_rust_prelude::AsTraitObject<
                super::super::rdfs::trait_object::ResourceDynTable<C>,
            > for B
        {
            fn as_trait_object(
                &self,
            ) -> (
                *const u8,
                super::super::rdfs::trait_object::ResourceDynTableInstance<C>,
            ) {
                let table =
                    super::super::rdfs::trait_object::ResourceDynTableInstance::<C>::new::<Self>();
                (self as *const Self as *const u8, table)
            }
        }
        impl<N: ::treeldr_rust_prelude::rdf_types::VocabularyMut>
            ::treeldr_rust_prelude::IntoJsonLdObjectMeta<N> for A
        where
            N: treeldr_rust_prelude::rdf_types::Namespace,
            N::Id: ::treeldr_rust_prelude::rdf_types::IntoId<Iri = N::Iri, BlankId = N::BlankId>,
            N::Iri: ::core::cmp::Eq + ::std::hash::Hash,
            N::BlankId: ::core::cmp::Eq + ::std::hash::Hash,
        {
            fn into_json_ld_object_meta(
                self,
                vocabulary: &mut N,
                meta: (),
            ) -> ::treeldr_rust_prelude::json_ld::IndexedObject<N::Iri, N::BlankId, ()>
            {
                let mut result = ::treeldr_rust_prelude::json_ld::Node::new();
                if let Some(value) = self.a {
                    result.properties_mut().insert(
                        ::treeldr_rust_prelude::locspan::Meta(
                            ::treeldr_rust_prelude::json_ld::Id::Valid(
                                ::treeldr_rust_prelude::json_ld::ValidId::Iri(
                                    vocabulary.insert(
                                        ::treeldr_rust_prelude::iref::Iri::new(
                                            "https://example.com/a",
                                        )
                                        .unwrap(),
                                    ),
                                ),
                            ),
                            (),
                        ),
                        ::treeldr_rust_prelude::IntoJsonLdObjectMeta::into_json_ld_object_meta(
                            value, vocabulary, meta,
                        ),
                    );
                }
                ::treeldr_rust_prelude::locspan::Meta(
                    ::treeldr_rust_prelude::json_ld::Indexed::new(
                        ::treeldr_rust_prelude::json_ld::Object::Node(Box::new(result)),
                        None,
                    ),
                    meta,
                )
            }
        }
        impl<I, C: ?Sized> super::super::rdfs::Resource<C> for A {
            type Type < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type Types < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type Label < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type Labels < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type Comment < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type Comments < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type SeeAlso < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type SeeAlsos < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            type IsDefinedBy < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
            type IsDefinedBys < 'a > = :: std :: iter :: Empty < & 'a :: std :: convert :: Infallible > where Self : 'a , C : 'a ;
            fn type_<'a>(&'a self, context: &'a C) -> Self::Types<'a> {
                ::std::iter::empty()
            }
            fn label<'a>(&'a self, context: &'a C) -> Self::Labels<'a> {
                ::std::iter::empty()
            }
            fn comment<'a>(&'a self, context: &'a C) -> Self::Comments<'a> {
                ::std::iter::empty()
            }
            fn see_also<'a>(&'a self, context: &'a C) -> Self::SeeAlsos<'a> {
                ::std::iter::empty()
            }
            fn is_defined_by<'a>(&'a self, context: &'a C) -> Self::IsDefinedBys<'a> {
                ::std::iter::empty()
            }
        }
        unsafe impl<I, C: ?Sized>
            ::treeldr_rust_prelude::AsTraitObject<
                super::super::rdfs::trait_object::ResourceDynTable<C>,
            > for A
        {
            fn as_trait_object(
                &self,
            ) -> (
                *const u8,
                super::super::rdfs::trait_object::ResourceDynTableInstance<C>,
            ) {
                let table =
                    super::super::rdfs::trait_object::ResourceDynTableInstance::<C>::new::<Self>();
                (self as *const Self as *const u8, table)
            }
        }
    }
    pub mod provider {
        pub trait AProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::A> {
            type A: super::A<C>;
            fn get(&self, id: &I) -> Option<&Self::A> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::A>>::get(self, id)
            }
        }
        pub trait BProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::B> {
            type B: super::B<C>;
            fn get(&self, id: &I) -> Option<&Self::B> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::B>>::get(self, id)
            }
        }
        pub trait EnumProvider<I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::Enum> {
            type Enum: super::Enum<C>;
            fn get(&self, id: &I) -> Option<&Self::Enum> {
                <Self as ::treeldr_rust_prelude::Provider<I, Self::Enum>>::get(self, id)
            }
        }
    }
    pub mod trait_object {
        pub struct BDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for BDynTable<C> {
            type Instance < 'a > = BDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct BDynTableInstance<'a, C: ?Sized> {
            pub b: unsafe fn(
                *const u8,
                context: ::treeldr_rust_prelude::ContravariantReference<'a, C>,
            ) -> Option<DynBB<'a, C>>,
        }
        impl<'a, C: ?Sized> Clone for BDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for BDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> BDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::B<C>>() -> Self {
                Self {
                    b: |ptr, context| unsafe {
                        let subject = &*(ptr as *const T);
                        let object = context.get(|context| subject.b(context));
                        object.map(DynBB::<'a, C>::new)
                    },
                }
            }
        }
        pub struct DynBB<'d, C: ?Sized> {
            _p: ::std::marker::PhantomData<&'d C>,
            ptr: *const u8,
            tables: (super::super::xsd::trait_object::StringDynTableInstance<C>,),
        }
        impl<'d, C: ?Sized> DynBB<'d, C> {
            pub fn new<T: super::super::xsd::String<C> + ::treeldr_rust_prelude::Reference<'d>>(
                value: T,
            ) -> Self {
                let ptr;
                let tables = ({
                    let (p, t) = ::treeldr_rust_prelude::AsTraitObject::<
                        super::super::xsd::trait_object::StringDynTable<C>,
                    >::into_trait_object(value);
                    ptr = p;
                    t
                },);
                Self {
                    _p: ::std::marker::PhantomData,
                    ptr,
                    tables,
                }
            }
        }
        impl<'d, C: ?Sized> Clone for DynBB<'d, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'d, C: ?Sized> Copy for DynBB<'d, C> {}
        impl<'d, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for DynBB<'d, C> {}
        impl<'d, C: ?Sized> super::super::xsd::String<C> for DynBB<'d, C> {}
        unsafe impl<'d, C: ?Sized>
            ::treeldr_rust_prelude::AsTraitObject<
                super::super::xsd::trait_object::StringDynTable<C>,
            > for DynBB<'d, C>
        {
            fn as_trait_object(
                &self,
            ) -> (
                *const u8,
                super::super::xsd::trait_object::StringDynTableInstance<C>,
            ) {
                (self.ptr, self.tables.0)
            }
            fn into_trait_object<'r>(
                self,
            ) -> (
                *const u8,
                super::super::xsd::trait_object::StringDynTableInstance<C>,
            )
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.0)
            }
        }
        pub struct ADynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for ADynTable<C> {
            type Instance < 'a > = ADynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct ADynTableInstance<'a, C: ?Sized> {
            pub a: unsafe fn(
                *const u8,
                context: ::treeldr_rust_prelude::ContravariantReference<'a, C>,
            ) -> Option<DynAA<'a, C>>,
        }
        impl<'a, C: ?Sized> Clone for ADynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for ADynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> ADynTableInstance<'a, C> {
            pub fn new<T: 'a + super::A<C>>() -> Self {
                Self {
                    a: |ptr, context| unsafe {
                        let subject = &*(ptr as *const T);
                        let object = context.get(|context| subject.a(context));
                        object.map(DynAA::<'a, C>::new)
                    },
                }
            }
        }
        pub struct DynAA<'d, C: ?Sized> {
            _p: ::std::marker::PhantomData<&'d C>,
            ptr: *const u8,
            tables: (super::super::xsd::trait_object::StringDynTableInstance<C>,),
        }
        impl<'d, C: ?Sized> DynAA<'d, C> {
            pub fn new<T: super::super::xsd::String<C> + ::treeldr_rust_prelude::Reference<'d>>(
                value: T,
            ) -> Self {
                let ptr;
                let tables = ({
                    let (p, t) = ::treeldr_rust_prelude::AsTraitObject::<
                        super::super::xsd::trait_object::StringDynTable<C>,
                    >::into_trait_object(value);
                    ptr = p;
                    t
                },);
                Self {
                    _p: ::std::marker::PhantomData,
                    ptr,
                    tables,
                }
            }
        }
        impl<'d, C: ?Sized> Clone for DynAA<'d, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'d, C: ?Sized> Copy for DynAA<'d, C> {}
        impl<'d, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for DynAA<'d, C> {}
        impl<'d, C: ?Sized> super::super::xsd::String<C> for DynAA<'d, C> {}
        unsafe impl<'d, C: ?Sized>
            ::treeldr_rust_prelude::AsTraitObject<
                super::super::xsd::trait_object::StringDynTable<C>,
            > for DynAA<'d, C>
        {
            fn as_trait_object(
                &self,
            ) -> (
                *const u8,
                super::super::xsd::trait_object::StringDynTableInstance<C>,
            ) {
                (self.ptr, self.tables.0)
            }
            fn into_trait_object<'r>(
                self,
            ) -> (
                *const u8,
                super::super::xsd::trait_object::StringDynTableInstance<C>,
            )
            where
                Self: ::treeldr_rust_prelude::Reference<'r>,
            {
                (self.ptr, self.tables.0)
            }
        }
        pub struct EnumDynTable<C: ?Sized>(std::marker::PhantomData<C>);
        impl<C: ?Sized> ::treeldr_rust_prelude::Table for EnumDynTable<C> {
            type Instance < 'a > = EnumDynTableInstance < 'a , C > where Self : 'a ;
        }
        pub struct EnumDynTableInstance<'a, C: ?Sized> {
            _d: ::std::marker::PhantomData<&'a C>,
        }
        impl<'a, C: ?Sized> Clone for EnumDynTableInstance<'a, C> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<'a, C: ?Sized> Copy for EnumDynTableInstance<'a, C> {}
        impl<'a, C: ?Sized> EnumDynTableInstance<'a, C> {
            pub fn new<T: 'a + super::Enum<C>>() -> Self {
                Self {
                    _d: ::std::marker::PhantomData,
                }
            }
        }
    }
    pub trait Enum<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::EnumDynTable<C>>
    {
    }
    impl<C: ?Sized> Enum<C> for ::std::convert::Infallible {}
    impl<'r, C: ?Sized, T: Enum<C>> Enum<C> for &'r T {}
    pub trait A<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::ADynTable<C>> + super::rdfs::Resource<C>
    {
        type A<'a>: ::treeldr_rust_prelude::Reference<'a> + super::xsd::String<C>
        where
            Self: 'a,
            C: 'a;
        fn a<'a>(&'a self, context: &'a C) -> Option<Self::A<'a>>;
    }
    impl<C: ?Sized> A<C> for ::std::convert::Infallible {
        type A < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
        fn a<'a>(&'a self, context: &'a C) -> Option<Self::A<'a>> {
            unreachable!()
        }
    }
    impl<'r, C: ?Sized, T: A<C>> A<C> for &'r T {
        type A < 'a > = T :: A < 'a > where Self : 'a , C : 'a ;
        fn a<'a>(&'a self, context: &'a C) -> Option<Self::A<'a>> {
            T::a(*self, context)
        }
    }
    pub trait B<C: ?Sized>:
        ::treeldr_rust_prelude::AsTraitObject<trait_object::BDynTable<C>> + super::rdfs::Resource<C>
    {
        type B<'a>: ::treeldr_rust_prelude::Reference<'a> + super::xsd::String<C>
        where
            Self: 'a,
            C: 'a;
        fn b<'a>(&'a self, context: &'a C) -> Option<Self::B<'a>>;
    }
    impl<C: ?Sized> B<C> for ::std::convert::Infallible {
        type B < 'a > = & 'a :: std :: convert :: Infallible where Self : 'a , C : 'a ;
        fn b<'a>(&'a self, context: &'a C) -> Option<Self::B<'a>> {
            unreachable!()
        }
    }
    impl<'r, C: ?Sized, T: B<C>> B<C> for &'r T {
        type B < 'a > = T :: B < 'a > where Self : 'a , C : 'a ;
        fn b<'a>(&'a self, context: &'a C) -> Option<Self::B<'a>> {
            T::b(*self, context)
        }
    }
}

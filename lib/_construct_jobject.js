exports._ = function _construct_jobject(callee, ...$args) {
    let c,
        r = new callee(...$args);
    if (callee === Proxy) {
        return r;
    }

    if (global.__jymfony !== void 0 && r instanceof __jymfony.JObject) {
        c = r.__construct(...$args);
    }

    if (
        void 0 !== global.mixins &&
        void 0 !== r[global.mixins.initializerSymbol]
    ) {
        r[global.mixins.initializerSymbol](...$args);
    }

    if (c !== void 0 && r !== c) {
        return c;
    }

    let self = r;
    if (Reflect.has(r, '__invoke')) {
        const proxy = new Proxy(self.__invoke, {
            get: (_, key) => {
                if (Symbol.for('jymfony.namespace.class') === key) {
                    return self;
                }

                return Reflect.get(self, key);
            },
            set(_, p, newValue, receiver) {
                return Reflect.set(self, p, newValue, receiver === proxy ? self : receiver);
            },
            has(_, p) {
                return Reflect.has(self, p);
            },
            deleteProperty(_, p) {
                return Reflect.deleteProperty(self, p);
            },
            defineProperty(_, property, attributes) {
                return Reflect.defineProperty(self, property, attributes);
            },
            enumerate(_) {
                return Reflect.enumerate(self);
            },
            ownKeys(_) {
                return Reflect.ownKeys(self);
            },
            apply: (_, ctx, args) => {
                return self.__invoke(...args);
            },
            construct(_, argArray, newTarget) {
                return Reflect.construct(self, argArray, newTarget);
            },
            getPrototypeOf(_) {
                return Reflect.getPrototypeOf(self);
            },
            setPrototypeOf(_, v) {
                return Reflect.setPrototypeOf(self, v);
            },
            isExtensible(_) {
                return Reflect.isExtensible(self);
            },
            preventExtensions(_) {
                Reflect.preventExtensions(self);

                return false;
            },
            getOwnPropertyDescriptor(target, key) {
                if (Symbol.for('jymfony.namespace.class') === key) {
                    return {
                        configurable: true,
                        enumerable: false,
                        value: self,
                    };
                }

                return Reflect.getOwnPropertyDescriptor(self, key);
            },
        });

        return proxy;
    }

    return self;
};

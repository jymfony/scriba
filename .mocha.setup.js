// Inject jest's assertion (expect) into global scope for the Mocha
// to use same assertion between node-swc & rest.
require('@jymfony/util');
global.expect = require('expect');
global.__jymfony.JObject = class {};

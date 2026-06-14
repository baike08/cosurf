const path = require('path');
const native = require(path.join(__dirname, 'native', 'cosurf_native.win32-x64-msvc.node'));

console.log('✅ Native module loaded successfully\n');
console.log('📋 Available methods:');

const methods = Object.getOwnPropertyNames(native).sort();
methods.forEach(method => {
  if (method.includes('user_event')) {
    console.log(`  [USER_EVENT] ${method}`);
  } else if (method.startsWith('db')) {
    console.log(`  [DB] ${method}`);
  }
});

console.log(`\nTotal methods: ${methods.length}`);
console.log('\n🔍 Checking user_events methods:');
const userEventMethods = methods.filter(m => m.includes('user_event'));
if (userEventMethods.length === 0) {
  console.log('  ❌ NO user_event methods found!');
} else {
  console.log(`  ✅ Found ${userEventMethods.length} user_event methods`);
}

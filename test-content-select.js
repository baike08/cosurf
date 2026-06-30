/**
 * 测试内容选择事件
 */

const { app } = require('electron');
const path = require('path');

// 模拟调用 Native 模块
async function testContentSelect() {
  try {
    // 加载 Native 模块
    const nativePath = path.join(__dirname, 'native', 'cosurf-native.node');
    console.log('Loading native module from:', nativePath);
    
    const native = require(nativePath);
    console.log('✅ Native module loaded');
    
    // 初始化数据库
    const dataDir = require('path').join(require('os').homedir(), 'AppData', 'Roaming', 'cosurf', 'cosurf-data');
    console.log('Initializing database at:', dataDir);
    native.nativeInit(dataDir, null); // 第二个参数为 null，让 Rust 自动从数据库读取
    console.log('✅ Database initialized');
    
    // 查询最近的内容选择事件
    console.log('\n📊 Querying content_select events...');
    const result = native.dbGetUserEvents(24, 10); // hours, limit
    
    const events = JSON.parse(result);
    console.log(`Found ${events.length} events`);
    
    // 过滤 content_select 类型的事件
    const contentSelectEvents = events.filter(e => e.type === 'content_select');
    console.log(`\n🎯 Content select events: ${contentSelectEvents.length}`);
    
    if (contentSelectEvents.length > 0) {
      console.log('\n--- Latest content selection ---');
      const latest = contentSelectEvents[0];
      console.log('ID:', latest.id);
      console.log('Type:', latest.type);
      console.log('URL:', latest.url);
      console.log('Timestamp:', new Date(latest.timestamp).toLocaleString());
      console.log('Selected text:', latest.data.selected_text?.substring(0, 100) + '...');
      console.log('Text length:', latest.data.text_length);
      console.log('Title:', latest.data.title);
    } else {
      console.log('\n⚠️  No content selection events found yet.');
      console.log('Please select some text in the browser to test.');
    }
    
  } catch (err) {
    console.error('❌ Error:', err.message);
    console.error(err.stack);
  }
}

testContentSelect();

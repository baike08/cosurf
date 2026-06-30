/**
 * 测试事件去重功能
 */

const path = require('path');

async function testDeduplication() {
  try {
    // 加载 Native 模块
    const nativePath = path.join(__dirname, 'native', 'cosurf-native.node');
    console.log('Loading native module from:', nativePath);
    
    const native = require(nativePath);
    console.log('✅ Native module loaded');
    
    // 初始化数据库
    const dataDir = path.join(require('os').homedir(), 'AppData', 'Roaming', 'cosurf', 'cosurf-data');
    console.log('Initializing database at:', dataDir);
    native.nativeInit(dataDir, null);
    console.log('✅ Database initialized\n');
    
    // 创建测试事件
    const testEvent = {
      id: `test-${Date.now()}`,
      type: 'content_select',
      timestamp: Date.now(),
      url: 'https://www.baidu.com',
      tab_id: 'test-tab',
      window_id: 1,
      data: {
        selected_text: '测试文本内容',
        text_length: 6,
        title: '测试页面',
        selection_type: 'text',
        highlight_color: '#ffeb3b'
      },
      created_at: Date.now()
    };
    
    console.log('📝 Inserting first event...');
    native.dbInsertUserEvent(JSON.stringify(testEvent));
    console.log('✅ First event inserted\n');
    
    // 等待 100ms
    await new Promise(resolve => setTimeout(resolve, 100));
    
    console.log('📝 Inserting duplicate event (same URL + text)...');
    const duplicateEvent = {
      ...testEvent,
      id: `test-duplicate-${Date.now()}`,
      timestamp: Date.now(),
      created_at: Date.now()
    };
    native.dbInsertUserEvent(JSON.stringify(duplicateEvent));
    console.log('✅ Duplicate event processed (should be skipped)\n');
    
    // 查询结果
    console.log('📊 Querying events...');
    const result = native.dbGetUserEvents(1, 10); // 最近 1 小时，最多 10 条
    const events = JSON.parse(result);
    
    // 只筛选测试事件（ID 以 test- 开头）
    const testEvents = events.filter(e => e.id.startsWith('test-'));
    console.log(`Found ${testEvents.length} test events (out of ${events.length} total)`);
    
    if (testEvents.length === 1) {
      console.log('\n✅ Deduplication working correctly!');
      console.log('Only 1 test event stored (duplicate was skipped)');
      
      const event = testEvents[0];
      console.log('\nEvent details:');
      console.log('  ID:', event.id);
      console.log('  Fingerprint:', event.fingerprint);
      console.log('  Text:', event.data.selected_text);
      console.log('  URL:', event.url);
    } else {
      console.log('\n❌ Deduplication NOT working!');
      console.log(`Expected 1 test event, but found ${testEvents.length}`);
      
      testEvents.forEach((e, i) => {
        console.log(`\nTest Event ${i + 1}:`);
        console.log('  ID:', e.id);
        console.log('  Fingerprint:', e.fingerprint);
      });
    }
    
  } catch (err) {
    console.error('❌ Error:', err.message);
    console.error(err.stack);
  }
}

testDeduplication();

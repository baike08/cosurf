import { useEffect, useRef, useState, useCallback } from "react";
import { Sparkles, AlertTriangle, RefreshCw, ExternalLink, Lock, Globe } from "lucide-react";
import { useTabStore } from "@/stores/tabStore";
import { getDomain } from "@/lib/utils";
import { tab as tabApi, shell as shellApi } from "@/lib/api";
import { on } from "@/lib/events";
import { ToolPage, parseToolUrl, isToolUrl } from "@/components/tools/ToolPage";
import { useHistoryStore } from "@/stores/historyStore";

/**
 * 注入链接拦截脚本,支持 target="_blank" 新开标签页
 * 同时阻止页面脚本调用 shell.open 导致的权限错误
 * 
 * 注意: 由于跨域限制,这个函数只对同源网站有效
 * 对于跨域网站,我们依赖 iframe 的 allow-popups 和 window.open 拦截
 */
function injectLinkInterceptor(doc: Document) {
  // 检查是否已经注入
  if ((doc as any).__cosurfLinkInterceptorInjected) {
    return;
  }
  (doc as any).__cosurfLinkInterceptorInjected = true;

  // 【关键】屏蔽 Tauri API,防止页面脚本调用 shell.open
  // 这可以阻止 biliMirror 等扩展脚本尝试打开外部链接
  if ((window as any).__TAURI__) {
    console.log('[LinkInterceptor] Blocking Tauri shell API to prevent unauthorized access');
    // 保存原始 API
    const originalTauri = (window as any).__TAURI__;
    
    // 创建一个安全的代理,只允许必要的功能
    (window as any).__TAURI__ = new Proxy(originalTauri, {
      get(target, prop) {
        // 阻止访问 shell API
        if (prop === 'shell') {
          console.warn('[LinkInterceptor] Blocked access to __TAURI__.shell');
          return undefined;
        }
        // 其他 API 正常访问
        return target[prop];
      }
    });
  }

  // 拦截所有链接点击(包括 target="_blank" 和普通链接)
  doc.addEventListener('click', (e: Event) => {
    const target = e.target as HTMLElement;
    const link = target.closest('a');
    
    if (link && link.href) {
      // 检测是否是外部链接或 target="_blank"
      const isExternal = link.target === '_blank' || 
                        link.rel?.includes('noopener') ||
                        (link.href.startsWith('http') && !link.href.includes(window.location.hostname));
      
      if (isExternal) {
        e.preventDefault();
        e.stopPropagation();
        
        console.log('[LinkInterceptor] Intercepted external link:', link.href);
        
        // 通过 postMessage 通知父窗口创建新标签页
        try {
          window.parent.postMessage({
            type: 'OPEN_NEW_TAB',
            url: link.href,
            source: 'link-click'
          }, '*');
          console.log('[LinkInterceptor] Sent OPEN_NEW_TAB message to parent');
        } catch (err) {
          console.error('[LinkInterceptor] Failed to send postMessage:', err);
        }
        
        return false;
      }
    }
  }, true); // 使用捕获阶段,优先于页面脚本执行

  // 覆盖 window.open 方法,防止调用 shell.open
  const originalOpen = window.open;
  window.open = function(url?: string | URL, target?: string, features?: string): WindowProxy | null {
    console.log('[LinkInterceptor] window.open called:', url, target);
    
    if (url) {
      // 将 URL 对象转换为字符串
      const urlString = typeof url === 'string' ? url : url.toString();
      
      try {
        // 阻止默认行为,改用 postMessage
        window.parent.postMessage({
          type: 'OPEN_NEW_TAB',
          url: urlString,
          source: 'window-open'
        }, '*');
        
        console.log('[LinkInterceptor] Redirected window.open to OPEN_NEW_TAB');
      } catch (err) {
        console.error('[LinkInterceptor] Failed to send postMessage from window.open:', err);
      }
      
      return null; // 返回 null 表示不打开新窗口
    }
    
    return originalOpen.call(window, url, target, features);
  };

  console.log('[LinkInterceptor] Injected successfully with shell.open protection');
}

/**
 * WebContentView - 使用 Tauri WebView2 加载网页
 * 支持所有网站正常加载
 */
export function WebContentView() {
  // 【关键修复】分别订阅 tabs 和 activeTabId，避免对象引用导致的无限循环
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  
  const updateTab = useTabStore((s) => s.updateTab);
  const addTab = useTabStore((s) => s.addTab);

  // 从最新的 tabs 数组中查找激活的标签页
  const activeTab = tabs.find((t: any) => t.id === activeTabId);
  
  // 【关键调试】详细日志
  console.log('[WebContentView] 🔍 Finding activeTab:', {
    activeTabId,
    tabCount: tabs.length,
    tabIds: tabs.map(t => t.id),
    foundActiveTab: !!activeTab,
    activeTabUrl: activeTab?.url,
    allTabs: tabs.map(t => ({ id: t.id, title: t.title, url: t.url }))
  });
  
  // 【关键修复】useEffect 必须在所有 return 之前调用
  // 监听 iframe 发来的消息(如新开标签页请求)
  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      if (event.data && event.data.type === 'OPEN_NEW_TAB') {
        const url = event.data.url;
        console.log('[WebContentView] Received OPEN_NEW_TAB message:', url);
        
        // 创建新标签页并导航到该 URL
        const newTabId = addTab(url, getDomain(url));
        console.log('[WebContentView] Created new tab:', newTabId);
      }
    };

    window.addEventListener('message', handleMessage);
    return () => window.removeEventListener('message', handleMessage);
  }, [addTab]);
  
  // 【新增】静默处理 iframe 中的 shell.open 错误
  useEffect(() => {
    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      // 检查是否是 shell.open 错误
      const errorMessage = event.reason?.message || String(event.reason);
      if (errorMessage.includes('shell.open not allowed')) {
        // 静默处理，不显示错误
        event.preventDefault();
        console.debug('[WebContentView] ✅ Suppressed shell.open error (Promise rejection)');
      }
    };
    
    const handleError = (event: ErrorEvent) => {
      // 检查是否是 shell.open 相关的错误
      if (event.message && event.message.includes('shell.open')) {
        event.preventDefault();
        console.debug('[WebContentView] ✅ Suppressed shell.open error (Error event)');
      }
    };
    
    window.addEventListener('unhandledrejection', handleUnhandledRejection);
    window.addEventListener('error', handleError);
    
    return () => {
      window.removeEventListener('unhandledrejection', handleUnhandledRejection);
      window.removeEventListener('error', handleError);
    };
  }, []);
  
  // 【新增】监听后端获取标签页信息的请求
  useEffect(() => {
    const unlisten = on<{ requestId: string; tabId: string }>(
      'webview:get-tab-info',
      async (payload) => {
        const { requestId, tabId } = payload;
        console.log('[WebContentView] 📥 Received get-tab-info request:', { requestId, tabId });
        
        // 查找对应的标签页
        const tab = tabs.find(t => t.id === tabId);
        
        if (tab) {
          console.log('[WebContentView] ✅ Found tab, sending response:', {
            url: tab.url,
            title: tab.title,
            isLoading: tab.isLoading
          });
          
          // 发送响应给后端
          try {
            await tabApi.getState(tabId);
            console.log('[WebContentView] 📤 Sent tab info response');
          } catch (error) {
            console.error('[WebContentView] ❌ Failed to send tab info:', error);
          }
        } else {
          console.warn('[WebContentView] ⚠️ Tab not found:', tabId);
        }
      }
    );
    
    return () => {
      unlisten();
    };
  }, [tabs]);
  
  // 【新增】监听后端获取标签页 URL 的请求
  useEffect(() => {
    const unlisten = on<{ tabId: string }>(
      'webview:get-tab-url',
      async (payload) => {
        const { tabId } = payload;
        console.log('[WebContentView] 📥 Received get-tab-url request:', { tabId });
        
        // 查找对应的标签页
        const tab = tabs.find(t => t.id === tabId);
        
        if (tab) {
          console.log('[WebContentView] ✅ Found tab, sending URL:', tab.url);
          
          // 发送响应给后端
          try {
            if (window.electronAPI) {
              window.electronAPI.send('cosurf:tab-url-response', { tabId, url: tab.url });
            }
            console.log('[WebContentView] 📤 Sent tab URL response');
          } catch (error) {
            console.error('[WebContentView] ❌ Failed to send URL response:', error);
          }
        } else {
          console.warn('[WebContentView] ⚠️ Tab not found:', tabId);
        }
      }
    );
    
    return () => {
      unlisten();
    };
  }, [tabs]);
  
  // 显示欢迎页的条件：没有活动标签且没有标签页
  if (!activeTab || tabs.length === 0) {
    console.log('[WebContentView] Showing welcome page', {
      activeTabId,
      tabCount: tabs.length,
      hasActiveTab: !!activeTab
    });
    return <WelcomePage />;
  }

  // 显示欢迎页的条件：没有活动标签
  // 注意：即使 URL 是 about:blank，也要渲染标签页列表，让 WebPageView 处理
  if (!activeTab) {
    console.log('[WebContentView] ⚠️ No active tab found, showing welcome page');
    return <WelcomePage />;
  }
  
  // 只渲染激活的标签页，其他标签页完全从 DOM 移除
  // 这样可以确保焦点始终在正确的 iframe 上
  return (
    <div className="flex-1 w-full flex flex-col min-h-0">
      <div
        key={activeTab.id}
        className="flex-1 w-full min-h-0"
        tabIndex={0}
        id={`tab-container-${activeTab.id}`}
      >
        {activeTab.url === "about:blank" ? (
          <WelcomePage />
        ) : isToolUrl(activeTab.url) ? (
          <ToolPage toolId={parseToolUrl(activeTab.url) || "unknown"} />
        ) : (
          <WebPageView
            tab={activeTab}
            onUpdateTab={(updates) => updateTab(activeTab.id, updates)}
          />
        )}
      </div>
    </div>
  );
}

function WelcomePage() {
  const activeTabId = useTabStore((s) => s.activeTabId);
  const updateTab = useTabStore((s) => s.updateTab);
  const [searchInput, setSearchInput] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  const navigateTo = (url: string) => {
    if (!activeTabId) return;
    updateTab(activeTabId, { url, title: getDomain(url), isLoading: true });
  };

  const handleSearch = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key !== 'Enter' || !searchInput.trim()) return;
    
    const input = searchInput.trim();
    
    // 判断是否是网址：包含点号且不含空格，或以 http/https 开头
    const isUrl = input.startsWith('http://') || input.startsWith('https://') ||
                  (input.includes('.') && !input.includes(' '));
    
    if (isUrl) {
      // 是网址，直接导航
      let url = input;
      if (!url.startsWith('http://') && !url.startsWith('https://')) {
        url = 'https://' + url;
      }
      navigateTo(url);
    } else {
      // 不是网址，使用百度搜索
      navigateTo(`https://www.baidu.com/s?wd=${encodeURIComponent(input)}`);
    }
    
    setSearchInput("");
  };

  // 自动聚焦输入框
  useEffect(() => {
    const timer = setTimeout(() => inputRef.current?.focus(), 100);
    return () => clearTimeout(timer);
  }, []);

  const quickLinks = [
    { name: "Google", url: "https://google.com", color: "bg-blue-500" },
    { name: "GitHub", url: "https://github.com", color: "bg-gray-700" },
    { name: "YouTube", url: "https://youtube.com", color: "bg-red-500" },
    { name: "Bilibili", url: "https://bilibili.com", color: "bg-pink-500" },
    { name: "知乎", url: "https://zhihu.com", color: "bg-blue-600" },
    { name: "百度", url: "https://baidu.com", color: "bg-gray-600" },
    { name: "Stack Overflow", url: "https://stackoverflow.com", color: "bg-amber-600" },
    { name: "MDN", url: "https://developer.mozilla.org", color: "bg-indigo-600" },
  ];

  return (
    <div className="h-full flex flex-col items-center justify-center gap-6 overflow-y-auto">
      <div className="flex flex-col items-center gap-2">
        <div className="w-20 h-20 rounded-3xl bg-gradient-to-br from-brand-500 to-brand-700 flex items-center justify-center shadow-lg shadow-brand-500/20">
          <Sparkles className="w-10 h-10 text-white" />
        </div>
        <h1 className="text-2xl font-bold text-content mt-2">
          欢迎使用 CoSurf
        </h1>
        <p className="text-sm text-content-secondary">
          你的 AI 阅读伴侣和思考搭档
        </p>
      </div>

      <div className="flex flex-col items-center gap-3 w-full max-w-lg px-6">
        {/* 搜索/网址输入框 */}
        <div className="w-full relative">
          <div className="flex items-center gap-2 h-11 rounded-xl px-4 bg-surface-secondary border border-border focus-within:border-brand-500 focus-within:ring-2 focus-within:ring-brand-500/20 transition-all">
            <Globe className="w-4 h-4 text-content-tertiary shrink-0" />
            <input
              ref={inputRef}
              type="text"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              onKeyDown={handleSearch}
              placeholder="输入网址或搜索内容，按回车访问"
              className="flex-1 bg-transparent text-sm text-content outline-none placeholder:text-content-tertiary"
              autoComplete="off"
            />
          </div>
          <div className="mt-1.5 text-2xs text-content-tertiary text-center">
            输入网址直接访问 · 其他内容使用百度搜索
          </div>
        </div>

        <div className="text-xs text-content-tertiary flex items-center gap-2">
          <span className="w-8 h-px bg-border" />
          快捷链接
          <span className="w-8 h-px bg-border" />
        </div>
      </div>

      <div className="grid grid-cols-4 gap-3 px-6">
        {quickLinks.map((site) => (
          <div
            key={site.name}
            onClick={() => navigateTo(site.url)}
            className="flex flex-col items-center gap-2 p-4 rounded-2xl hover:bg-surface-hover cursor-pointer transition-colors group"
          >
            <div
              className={`w-12 h-12 rounded-xl ${site.color} flex items-center justify-center text-white text-lg font-bold shadow-md group-hover:scale-105 transition-transform`}
            >
              {site.name.charAt(0)}
            </div>
            <span className="text-xs text-content-secondary group-hover:text-content transition-colors">
              {site.name}
            </span>
          </div>
        ))}
      </div>

      <div className="mt-4 px-6">
        <div className="flex items-center gap-2 text-2xs text-content-tertiary">
          <Sparkles className="w-3 h-3" />
          <span>按 Ctrl+L 聚焦地址栏 · AI 对话面板随时唤醒</span>
        </div>
      </div>
    </div>
  );
}

function WebPageView({
  tab,
  onUpdateTab,
}: {
  tab: { id: string; title: string; url: string; isLoading: boolean };
  onUpdateTab: (updates: { title?: string; url?: string; isLoading?: boolean; canGoBack?: boolean; canGoForward?: boolean }) => void;
}) {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const webviewRef = useRef<any>(null);
  const [loadError, setLoadError] = useState(false);
  const [securityInfo, setSecurityInfo] = useState<{ secure: boolean; domain: string } | null>(null);
  const loadTimeoutRef = useRef<number | null>(null);
  
  // 获取活跃标签页 ID
  const activeTabId = useTabStore((s) => s.activeTabId);
  
  // 监听 <webview> 事件
  useEffect(() => {
    const webview = webviewRef.current;
    if (!webview) return;
    
    console.log('[WebPageView] 🔧 Setting up webview event listeners');
    
    const handleDidStartLoading = () => {
      console.log('[WebPageView] 🚀 WebView started loading:', tab.url);
      onUpdateTab({ isLoading: true });
    };
    
    const handleDidStopLoading = () => {
      console.log('[WebPageView] ✅ WebView stopped loading:', tab.url);
      onUpdateTab({ isLoading: false });
      setLoadError(false);
    };
    
    const handlePageTitleUpdated = (e: any) => {
      console.log('[WebPageView] 📝 Page title updated:', e.title);
      if (e.title) {
        onUpdateTab({ title: e.title });
      }
    };
    
    const handleDidNavigate = (e: any) => {
      console.log('[WebPageView] 🧭 Navigated to:', e.url);
      onUpdateTab({ url: e.url, isLoading: false });
    };
    
    const handleNewWindow = (e: any) => {
      console.log('[WebPageView] 🆕 New window request:', e.url);
      // 阻止默认行为（不在外部浏览器打开）
      e.preventDefault();
      
      // 通过 IPC 通知主进程创建新标签页
      window.electron?.ipcRenderer?.send('webview:create-tab', {
        url: e.url,
        title: '加载中...',
      });
    };
    
    webview.addEventListener('did-start-loading', handleDidStartLoading);
    webview.addEventListener('did-stop-loading', handleDidStopLoading);
    webview.addEventListener('page-title-updated', handlePageTitleUpdated);
    webview.addEventListener('did-navigate', handleDidNavigate);
    webview.addEventListener('new-window', handleNewWindow);
    
    return () => {
      webview.removeEventListener('did-start-loading', handleDidStartLoading);
      webview.removeEventListener('did-stop-loading', handleDidStopLoading);
      webview.removeEventListener('page-title-updated', handlePageTitleUpdated);
      webview.removeEventListener('did-navigate', handleDidNavigate);
      webview.removeEventListener('new-window', handleNewWindow);
    };
  }, [tab.url, onUpdateTab]);
  
  // 当标签页激活时，聚焦 iframe
  useEffect(() => {
    if (tab.id === activeTabId) {
      console.log('[WebPageView] 🔄 Tab activated:', tab.id, 'URL:', tab.url);
        
      // 【核心改进】使用 requestAnimationFrame 确保 DOM 更新后立即聚焦
      const focusInNextFrame = () => {
        requestAnimationFrame(() => {
          performFocus();
        });
      };
        
      // 定义聚焦函数
      const performFocus = () => {
        console.log('[WebPageView] 🎯 Performing focus for tab:', tab.id);
          
        // 1. 首先确保浏览器窗口获得焦点（最关键！）
        window.focus();
        console.log('[WebPageView] ✅ Focused main window');
          
        // 2. 等待一小段时间让 window.focus() 生效
        setTimeout(() => {
          // 3. 聚焦容器 div
          const container = document.getElementById(`tab-container-${tab.id}`);
          if (container) {
            container.focus();
            console.log('[WebPageView] ✅ Focused tab container');
          }
            
          // 4. 聚焦 iframe 元素
          const iframe = iframeRef.current;
          if (iframe) {
            iframe.focus();
            console.log('[WebPageView] ✅ Focused iframe element');
              
            // 5. 尝试聚焦 iframe 的内容窗口（同源时有效）
            try {
              if (iframe.contentWindow) {
                iframe.contentWindow.focus();
                console.log('[WebPageView] ✅ Focused iframe content window');
              }
            } catch (e) {
              console.log('[WebPageView] ⚠️ Cannot focus content window (cross-origin):', e);
            }
          }
        }, 50); // 50ms 延迟确保 window.focus() 生效
      };
        
      // 如果页面已经加载完成，立即聚焦
      if (!tab.isLoading && tab.url !== 'about:blank') {
        console.log('[WebPageView] ✅ Page already loaded, focusing immediately');
        focusInNextFrame();
      } else {
        console.log('[WebPageView] ⏳ Page is loading or about:blank, waiting for load event');
          
        // 监听 iframe 的 load 事件，在加载完成后聚焦
        const iframe = iframeRef.current;
        if (iframe) {
          const handleLoad = () => {
            console.log('[WebPageView] 📥 iframe load event triggered, performing focus');
            // 等待一小段时间让页面完全渲染
            setTimeout(focusInNextFrame, 100);
          };
            
          iframe.addEventListener('load', handleLoad, { once: true });
            
          // 设置超时保护：如果 5 秒后还没触发 load 事件，也执行聚焦
          const timeoutId = setTimeout(() => {
            console.log('[WebPageView] ⏰ Load timeout, performing focus anyway');
            focusInNextFrame();
          }, 5000);
            
          // 清理函数
          return () => {
            iframe.removeEventListener('load', handleLoad);
            clearTimeout(timeoutId);
          };
        }
      }
        
      // 如果标题还是"加载中..."，尝试获取真实标题
      if (tab.title === '加载中...' || tab.title === '新标签页') {
        console.log('[WebPageView] 🔄 Title is still loading, requesting from backend');
        const requestTitle = async () => {
          try {
            const title = await tabApi.getTitle(tab.id);
            console.log('[WebPageView] 📥 Backend response:', title);
            if (title && title !== '加载中...' && !title.includes('未知')) {
              onUpdateTab({ title });
              console.log('[WebPageView] ✅ Got title from backend:', title);
            } else {
              // 使用主机名作为后备
              const urlObj = new URL(tab.url);
              const hostname = urlObj.hostname.replace('www.', '');
              const fallbackTitle = hostname.charAt(0).toUpperCase() + hostname.slice(1);
              onUpdateTab({ title: fallbackTitle });
              console.log('[WebPageView] Using hostname as fallback:', fallbackTitle);
            }
          } catch (error) {
            console.error('[WebPageView] Failed to get title from backend:', error);
            // 使用主机名作为最后的后备
            try {
              const urlObj = new URL(tab.url);
              const hostname = urlObj.hostname.replace('www.', '');
              const fallbackTitle = hostname.charAt(0).toUpperCase() + hostname.slice(1);
              onUpdateTab({ title: fallbackTitle });
            } catch (urlError) {
              console.warn('[WebPageView] Failed to parse URL:', urlError);
            }
          }
        };
        requestTitle();
      }
    }
  }, [tab.id, activeTabId, tab.isLoading, tab.title, tab.url]);

  // 获取安全信息
  useEffect(() => {
    try {
      const url = new URL(tab.url);
      setSecurityInfo({
        secure: url.protocol === "https:",
        domain: url.hostname,
      });
    } catch {
      setSecurityInfo(null);
    }
  }, [tab.url]);

  // 设置加载超时检测
  useEffect(() => {
    if (tab.isLoading) {
      // 清除之前的超时
      if (loadTimeoutRef.current) {
        clearTimeout(loadTimeoutRef.current);
      }
      
      // 设置10秒超时
      loadTimeoutRef.current = setTimeout(() => {
        console.log('[WebPageView] Load timeout detected for:', tab.url);
        // 检查 iframe 是否真的加载成功
        try {
          const iframe = iframeRef.current;
          if (iframe && iframe.contentDocument && iframe.contentDocument.body) {
            // 如果能访问 contentDocument，说明加载成功了（同源）
            return;
          }
        } catch (e) {
          // 跨域错误，这很正常，不视为失败
        }
        
        // 对于跨域网站，我们无法检测是否加载成功，所以不显示错误
        // 只有当 iframe 完全无法加载时才显示错误
      }, 10000);
    }
    
    return () => {
      if (loadTimeoutRef.current) {
        clearTimeout(loadTimeoutRef.current);
      }
    };
  }, [tab.isLoading, tab.url]);

  // 【新增】监听 webview 加载完成事件，通知主进程进行内容提取
  useEffect(() => {
    const webview = webviewRef.current;
    if (!webview) return;
    
    const handleDidFinishLoad = () => {
      console.log('[WebPageView] 📥 Webview finished loading:', webview.getURL());
      
      // 发送加载完成事件给主进程
      if (window.electronAPI) {
        window.electronAPI.send('webview:did-finish-load', {
          tabId: tab.id,
          url: webview.getURL(),
          title: document.title || webview.getTitle() || '',
        });
      }
    };
    
    webview.addEventListener('did-finish-load', handleDidFinishLoad);
    
    return () => {
      webview.removeEventListener('did-finish-load', handleDidFinishLoad);
    };
  }, [tab.id]);

  // 监听后端发来的导航事件
  useEffect(() => {
    const unlistenNavigating = on<any>('webview:navigating', (payload) => {
      const { tabId, url } = payload;
      if (tabId === tab.id && iframeRef.current) {
        console.log('[WebPageView] Received navigate event:', url);
        iframeRef.current.src = url;
        // 更新标签页状态：URL、加载状态
        onUpdateTab({ 
          url, 
          isLoading: true,
          title: '加载中...'
        });
      }
    });
  
    const unlistenReload = on<any>('webview:reload', (payload) => {
      const { tabId } = payload;
      if (tabId === tab.id && iframeRef.current) {
        console.log('[WebPageView] Received reload event');
        iframeRef.current.src = iframeRef.current.src;
      }
    });
  
    // 监听获取页面内容事件（用于AI总结）
    const unlistenGetContent = on<any>('webview:get-content', async (payload) => {
      console.log('[WebPageView] 📥 Received webview:get-content event:', payload);
      const { tabId, script, requestId: _requestId } = payload;
      console.log('[WebPageView] 🔍 Checking tab match:', {
        receivedTabId: tabId,
        currentTabId: tab.id,
        matches: tabId === tab.id,
        hasIframe: !!iframeRef.current
      });
        
      if (tabId === tab.id && iframeRef.current) {
        console.log('[WebPageView] ✅ Tab ID matches, processing request');
        try {
          const iframeDoc = iframeRef.current.contentDocument || iframeRef.current.contentWindow?.document;
          if (!iframeDoc) {
            console.warn('[WebPageView] ❌ Cannot access iframe content - cross-origin restriction');
            // 发送错误响应
            await tabApi.getState(tabId).catch((err: any) => {
              console.error('[WebPageView] Failed to send error response:', err);
            });
            return;
          }
            
          console.log('[WebPageView] ✅ Can access iframe content, executing script');
          // 执行脚本获取页面内容
          const result = iframeDoc.defaultView?.eval(script);
          console.log('[WebPageView] ✅ Script executed, result length:', result?.length);
            
          // 通过自定义事件返回结果
          window.dispatchEvent(new CustomEvent('cosurf:page-content', {
            detail: { tabId, content: result }
          }));
            
          // 发送响应给后端
          console.log('[WebPageView] 📤 Sending page content response to backend...');
          await tabApi.getState(tabId);
          console.log('[WebPageView] ✅ Sent page content response to backend');
        } catch (error) {
          console.error('[WebPageView] ❌ Failed to extract page content:', error);
          // 跨域限制，尝试使用其他方法
          window.dispatchEvent(new CustomEvent('cosurf:page-content-error', {
            detail: { tabId, error: String(error) }
          }));
            
          // 发送错误响应
          console.log('[WebPageView] 📤 Sending error response to backend...');
          await tabApi.getState(tabId).catch((err: any) => {
            console.error('[WebPageView] Failed to send error response:', err);
          });
        }
      } else {
        console.warn('[WebPageView] ⚠️ Tab ID mismatch or iframe not ready:', {
          receivedTabId: tabId,
          currentTabId: tab.id,
          hasIframe: !!iframeRef.current
        });
      }
    });
  
    // 【新增】监听后端发来的内容提取请求
    const unlistenExtractContent = on<any>('webview:extract-content', async (payload) => {
      const { tabId, url } = payload;
      console.log('[WebPageView] 📥 Received extract-content request:', { tabId, url });
        
      if (tabId !== tab.id) {
        console.log('[WebPageView] ⏭️  Tab ID mismatch, skipping');
        return;
      }
        
      const webview = webviewRef.current;
      if (!webview) {
        console.warn('[WebPageView] ❌ Webview not ready');
        return;
      }
        
      try {
        console.log('[WebPageView] 🚀 Injecting Readability script...');
          
        // 注入 Readability 并执行提取
        const result = await webview.executeJavaScript(`
          (async function() {
            try {
              // 1. 加载 Readability
              if (!window.Readability) {
                const script = document.createElement('script');
                script.src = 'https://cdn.jsdelivr.net/npm/@mozilla/readability@0.4.4/Readability.min.js';
                document.head.appendChild(script);
                await new Promise(resolve => script.onload = resolve);
              }
                
              // 2. 执行提取
              const doc = new Readability(document.cloneNode(true), {
                charThreshold: 500,
                keepClasses: false,
                nbTopCandidates: 5,
              });
                
              const article = doc.parse();
                
              if (!article || !article.content) {
                return JSON.stringify({ error: 'No readable content found' });
              }
                
              // 3. 返回结果
              return JSON.stringify({
                title: article.title || document.title,
                content: article.content,
                excerpt: article.excerpt || '',
              });
            } catch (err) {
              return JSON.stringify({ error: err.message });
            }
          })();
        `, true); // true 表示在用户手势上下文中执行
          
        console.log('[WebPageView] ✅ Readability extraction completed');
          
        // 解析结果
        const article = JSON.parse(result);
          
        if (article.error) {
          console.warn('[WebPageView] ⚠️  Extraction failed:', article.error);
          return;
        }
          
        // 发送到主进程
        console.log('[WebPageView] 📤 Sending extracted content to backend...');
        if (window.electronAPI) {
          window.electronAPI.send('webview:content-extracted', {
            url: webview.getURL(),
            tabId: tab.id,
            title: article.title,
            content: article.content,
            excerpt: article.excerpt,
          });
        }
          
        console.log('[WebPageView] ✅ Content sent to backend');
      } catch (err) {
        console.error('[WebPageView] ❌ Failed to inject Readability:', err);
      }
    });
  
    return () => {
      unlistenNavigating();
      unlistenReload();
      unlistenGetContent();
      unlistenExtractContent();
    };
  }, [tab.id, onUpdateTab]);

  // 监听元素选择事件
  useEffect(() => {
    const unlistenElementSelected = on<any>('element-selected', (payload) => {
      const { selector } = payload;
      console.log('[WebPageView] Element selected:', selector);
      // 这里可以通过自定义事件通知 BrowserActionPanel
      window.dispatchEvent(new CustomEvent('cosurf:element-selected', { detail: { selector } }));
    });

    return () => {
      unlistenElementSelected();
    };
  }, []);

  // 处理 iframe 加载事件
  const handleIframeLoad = useCallback(() => {
    onUpdateTab({ isLoading: false });
    setLoadError(false);

    // 记录浏览历史
    if (tab.url && tab.url !== 'about:blank' && !tab.url.startsWith('cosurf://')) {
      const title = tab.title || getDomain(tab.url);
      useHistoryStore.getState().addHistory(title, tab.url);
    }
      
    // 尝试获取标题和注入脚本
    try {
      const iframe = iframeRef.current;
      if (iframe && iframe.contentDocument) {
        const title = iframe.contentDocument.title;
        if (title) {
          onUpdateTab({ title });
          console.log('[WebPageView] ✅ Got title from iframe:', title);
        }
          
        // 注入脚本拦截链接点击,支持 target="_blank"
        injectLinkInterceptor(iframe.contentDocument);
        console.log('[WebPageView] ✅ Injected link interceptor (same-origin)');
      }
    } catch (e) {
      // 跨域限制,这是正常的
      console.log('[WebPageView] ⚠️ Cannot access iframe content (cross-origin)');
      console.log('[WebPageView] ℹ️ This is a cross-origin website, link clicks will use browser default behavior');
      console.log('[WebPageView]');
      console.log('[WebPageView] 💡 How to open links in CoSurf:');
      console.log('[WebPageView]    1. Right-click the link → "Open link in new tab" (if available)');
      console.log('[WebPageView]    2. Copy link URL → Paste in CoSurf address bar');
      console.log('[WebPageView]    3. Use AI Agent: Type "open [url] in new tab"');
      console.log('[WebPageView]    4. Use AI Agent: Type "click the [button name] button"');
      console.log('[WebPageView]');
      console.log('[WebPageView] 📖 Learn more: docs/iframe-link-click-guide.md');
      
      // 对于跨域网站,请求后端获取真实标题
      const requestTitle = async () => {
        try {
          const title = await tabApi.getTitle(tab.id);
          console.log('[WebPageView] 📥 Backend response:', title);
          if (title && title !== '加载中...' && !title.includes('未知')) {
            onUpdateTab({ title });
            console.log('[WebPageView] ✅ Got title from backend:', title);
          } else {
            // 如果后端也无法获取，使用主机名作为后备
            const urlObj = new URL(tab.url);
            const hostname = urlObj.hostname.replace('www.', '');
            const fallbackTitle = hostname.charAt(0).toUpperCase() + hostname.slice(1);
            onUpdateTab({ title: fallbackTitle });
            console.log('[WebPageView] Using hostname as fallback:', fallbackTitle);
          }
        } catch (error) {
          console.error('[WebPageView] Failed to get title from backend:', error);
          // 使用主机名作为最后的后备
          try {
            const urlObj = new URL(tab.url);
            const hostname = urlObj.hostname.replace('www.', '');
            const fallbackTitle = hostname.charAt(0).toUpperCase() + hostname.slice(1);
            onUpdateTab({ title: fallbackTitle });
          } catch (urlError) {
            console.warn('[WebPageView] Failed to parse URL:', urlError);
          }
        }
      };
      
      requestTitle();
      
      // 对于跨域网站,我们依赖后端事件系统来处理新标签页
      // 这里不需要做任何事,因为 postMessage 仍然可以工作
    }
  }, [onUpdateTab, tab.id, tab.url]);

  const handleIframeError = useCallback(() => {
    console.log('[WebPageView] iframe load error');
    setLoadError(true);
    onUpdateTab({ isLoading: false });
  }, [onUpdateTab]);

  // 添加超时检测,处理 X-Frame-Options 限制
  useEffect(() => {
    if (tab.isLoading && tab.url !== 'about:blank') {
      const timeout = setTimeout(() => {
        console.log('[WebPageView] Load timeout check for:', tab.url);
        
        // 检查是否能访问 iframe 内容
        try {
          const iframe = iframeRef.current;
          if (iframe) {
            // 尝试访问 contentDocument
            const doc = iframe.contentDocument || iframe.contentWindow?.document;
            if (doc && doc.body && doc.body.innerHTML.length > 0) {
              // 能访问且有内容,说明加载成功了
              console.log('[WebPageView] Page loaded successfully');
              return;
            }
          }
        } catch (e) {
          // 跨域错误,这可能是正常的,也可能意味着被阻止
          const domain = getDomain(tab.url);
          
          // 已知的会阻止 iframe 的网站列表
          const blockedDomains = [
            'zhihu.com', 'www.zhihu.com',
            'taobao.com', 'www.taobao.com',
            'jd.com', 'www.jd.com',
            'baidu.com', 'www.baidu.com',
            'weibo.com', 'www.weibo.com',
            'facebook.com', 'www.facebook.com',
            'twitter.com', 'www.twitter.com',
            'douyin.com', 'www.douyin.com',
            'youtube.com', 'www.youtube.com',
            'netflix.com', 'www.netflix.com',
            'instagram.com', 'www.instagram.com',
            'tiktok.com', 'www.tiktok.com',
            'reddit.com', 'www.reddit.com',
          ];
          
          const isBlocked = blockedDomains.some(d => domain.includes(d));
          
          if (isBlocked) {
            console.log('[WebPageView] 🚫 Detected blocked domain:', domain);
            setLoadError(true);
            onUpdateTab({ isLoading: false });
            return;
          }
          
          // 对于其他网站,跨域是正常的,不显示错误
          console.log('[WebPageView] Cross-origin page (normal)');
        }
        
        // 如果30秒后还在加载状态,可能是网络问题或被 CSP 阻止
        if (tab.isLoading) {
          console.log('[WebPageView] ⚠️ Still loading after 30s, showing error');
          setLoadError(true);
          onUpdateTab({ isLoading: false });
        }
      }, 30000); // 从 8 秒延长到 30 秒

      return () => clearTimeout(timeout);
    }
  }, [tab.isLoading, tab.url, onUpdateTab]);

  // 刷新页面
  const handleReload = useCallback(() => {
    if (iframeRef.current) {
      iframeRef.current.src = iframeRef.current.src;
    }
  }, []);

  // 后退 - iframe 不支持历史记录 API，暂时禁用
  const handleGoBack = useCallback(() => {
    console.log("Go back not supported for iframe");
  }, []);

  // 前进 - iframe 不支持历史记录 API，暂时禁用
  const handleGoForward = useCallback(() => {
    console.log("Go forward not supported for iframe");
  }, []);

  // 暴露导航方法给父组件
  useEffect(() => {
    if (iframeRef.current) {
      (iframeRef.current as any).__browserNav = {
        goBack: handleGoBack,
        goForward: handleGoForward,
        reload: handleReload,
        navigate: (url: string) => {
          if (iframeRef.current) {
            iframeRef.current.src = normalizeUrl(url);
            onUpdateTab({ isLoading: true });
          }
        },
      };
    }
  }, [handleGoBack, handleGoForward, handleReload, onUpdateTab]);

  const openExternal = () => {
    shellApi.openUrl(tab.url);
  };

  return (
    <div className="h-full w-full flex flex-col relative">
      {/* 加载进度条 */}
      {tab.isLoading && (
        <div className="absolute top-0 left-0 right-0 z-20 h-0.5 bg-surface-secondary">
          <div className="h-full bg-gradient-to-r from-brand-500 via-brand-400 to-brand-500 animate-loading-bar" />
        </div>
      )}

      {/* 安全信息条 */}
      {securityInfo && !tab.isLoading && (
        <div className="flex items-center gap-1.5 px-3 py-0.5 bg-surface-secondary/50 border-b border-border/50 text-2xs text-content-tertiary shrink-0">
          {securityInfo.secure ? (
            <Lock className="w-3 h-3 text-green-500" />
          ) : (
            <Globe className="w-3 h-3 text-amber-500" />
          )}
          <span>{securityInfo.domain}</span>
        </div>
      )}

      {/* iframe 容器 */}
      {loadError ? (
        <div className="flex-1 flex flex-col items-center justify-center gap-4 text-content-tertiary">
          <AlertTriangle className="w-12 h-12 text-amber-500" />
          <div className="text-center">
            <div className="text-sm font-medium text-content-secondary">
              页面加载失败
            </div>
            <div className="text-xs mt-1 text-content-tertiary max-w-xs">
              {getDomain(tab.url)}
            </div>
            <div className="text-2xs mt-2 text-content-tertiary max-w-md">
              该网站设置了安全策略（CSP/X-Frame-Options），禁止在 iframe 中嵌入。
              <br />
              常见被阻止的网站：百度、抖音、知乎、淘宝、YouTube、Facebook 等。
            </div>
          </div>
          <div className="flex gap-2 mt-2">
            <button
              onClick={handleReload}
              className="flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs border border-border hover:bg-surface-hover transition-colors"
            >
              <RefreshCw className="w-3.5 h-3.5" />
              重试
            </button>
            <button
              onClick={openExternal}
              className="flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs bg-brand-600 text-white hover:bg-brand-700 transition-colors"
            >
              <ExternalLink className="w-3.5 h-3.5" />
              在系统浏览器中打开
            </button>
          </div>
        </div>
      ) : (
        // 使用 <webview> tag 替代 iframe，解决 CSP/X-Frame-Options 问题
        // webSecurity=no 禁用同源策略和 CSP 检查
        // 注意：preload 通过 session.setPreloads() 在主进程中配置
        <webview
          ref={webviewRef}
          id={`webview-${tab.id}`}
          src={normalizeUrl(tab.url)}
          className="flex-1 w-full border-0"
          allowpopups
          plugins
          nodeintegration={false as any}
          partition="persist:cosurf-webview"
          webpreferences="contextIsolation=yes nodeIntegration=no webSecurity=no allowRunningInsecureContent=yes nativeWindowOpen=yes"
        />
      )}
    </div>
  );
}

function normalizeUrl(input: string): string {
  if (input.startsWith("http://") || input.startsWith("https://")) {
    return input;
  }
  if (input.includes(".") && !input.includes(" ")) {
    return "https://" + input;
  }
  return "https://www.baidu.com/s?wd=" + encodeURIComponent(input);
}

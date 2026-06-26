document.addEventListener('DOMContentLoaded', () => {
    const searchInput = document.getElementById('searchInput');
    const searchBtn = document.getElementById('searchBtn');
    const engineBtns = document.querySelectorAll('.engine-btn');
    const zenModeBtn = document.getElementById('zenModeBtn');
    const wallpaperModeBtn = document.getElementById('wallpaperModeBtn');

    // 壁纸清晰度切换功能
    function initWallpaperMode() {
        const isClearWallpaper = localStorage.getItem('clear_wallpaper') === 'true';
        if (isClearWallpaper) {
            document.body.classList.add('clear-wallpaper');
            console.log('Wallpaper: clear mode restored');
        }

        if (wallpaperModeBtn) {
            wallpaperModeBtn.addEventListener('click', (e) => {
                e.preventDefault();
                e.stopPropagation();
                document.body.classList.toggle('clear-wallpaper');
                const isNowClear = document.body.classList.contains('clear-wallpaper');
                localStorage.setItem('clear_wallpaper', isNowClear);
                console.log('Wallpaper:', isNowClear ? 'CLEAR' : 'BLURRED');
            });
            console.log('Wallpaper: button initialized');
        }
    }
    initWallpaperMode();

    // Zen Mode 功能
    function initZenMode() {
        // 默认开启 Zen Mode：未保存过偏好时（null）默认为 true，否则尊重用户选择
        const storedZen = localStorage.getItem('zen_mode');
        const isZenMode = storedZen === null ? true : storedZen === 'true';
        if (isZenMode) {
            document.body.classList.add('zen-mode');
            console.log('Zen mode: default/restored ->', isZenMode);
        }

        if (zenModeBtn) {
            zenModeBtn.addEventListener('click', (e) => {
                e.preventDefault();
                e.stopPropagation();
                document.body.classList.toggle('zen-mode');
                const isNowZenMode = document.body.classList.contains('zen-mode');
                localStorage.setItem('zen_mode', isNowZenMode);
                if (isNowZenMode) document.body.classList.remove('reveal-search');
                console.log('Zen mode:', isNowZenMode ? 'ON' : 'OFF');
            });
            console.log('Zen mode: button initialized');
        } else {
            console.error('Zen mode: button not found!');
        }
    }
    initZenMode();

    // Zen Mode 下搜索框的"鼠标唤出"：默认隐藏；鼠标移动/点击/触摸时淡入，
    // 静止 2.5s 后淡出（正在输入时不隐藏）。
    function initZenReveal() {
        let hideTimer;
        const IDLE_MS = 2500;
        const armHide = () => {
            clearTimeout(hideTimer);
            hideTimer = setTimeout(() => {
                if (document.activeElement !== searchInput) {
                    document.body.classList.remove('reveal-search');
                }
            }, IDLE_MS);
        };
        const reveal = () => {
            if (!document.body.classList.contains('zen-mode')) return;
            document.body.classList.add('reveal-search');
            armHide();
        };
        document.addEventListener('mousemove', reveal);
        document.addEventListener('mousedown', reveal);
        document.addEventListener('touchstart', reveal, { passive: true });
        if (searchInput) {
            // 聚焦时保持可见；失焦后重新计时淡出
            searchInput.addEventListener('focus', () => {
                clearTimeout(hideTimer);
                document.body.classList.add('reveal-search');
            });
            searchInput.addEventListener('blur', armHide);
        }
    }
    initZenReveal();

    // 分类卡片展开/收起：超过 6 条的链接默认收起，点击按钮展开
    function initCategoryToggles() {
        document.querySelectorAll('.cat-toggle').forEach(btn => {
            btn.addEventListener('click', (e) => {
                e.preventDefault();
                e.stopPropagation();
                const card = btn.closest('.glass');
                if (!card) return;
                const overflow = card.querySelectorAll('.cat-overflow');
                const expanded = btn.dataset.expanded === 'true';
                // expanded=false -> 当前收起，点击应展开（移除 hidden）
                overflow.forEach(li => li.classList.toggle('hidden', expanded));
                btn.dataset.expanded = (!expanded).toString();
                const textEl = btn.querySelector('.cat-toggle-text');
                const iconEl = btn.querySelector('.cat-toggle-icon');
                if (textEl) textEl.textContent = expanded ? `展开全部 ${btn.dataset.count} 个` : '收起';
                if (iconEl) iconEl.classList.toggle('rotate-180', !expanded);
            });
        });
    }
    initCategoryToggles();

    // 获取当前激活的搜索引擎 URL
    function getActiveEngineUrl() {
        const activeBtn = document.querySelector('.engine-btn[data-active="true"]');
        return activeBtn ? activeBtn.dataset.url : '';
    }

    // 设置激活的搜索引擎
    function setActiveEngine(btn) {
        // 移除所有激活状态
        engineBtns.forEach(b => {
            b.removeAttribute('data-active');
            b.className = 'engine-btn px-4 py-2 rounded-full text-sm font-medium transition-all duration-300 glass text-gray-300 hover:text-white hover:bg-white/10';
        });

        // 添加激活状态
        btn.setAttribute('data-active', 'true');
        btn.className = 'engine-btn px-4 py-2 rounded-full text-sm font-medium transition-all duration-300 bg-gradient-to-r from-primary to-secondary text-white shadow-lg shadow-primary/30 scale-105';

        // 保存偏好
        localStorage.setItem('preferred_engine', btn.dataset.url);
    }

    // 搜索引擎切换
    engineBtns.forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            setActiveEngine(btn);
            searchInput.focus();
        });
    });

    // 初始化：用回退链选出唯一目标引擎，再统一激活，确保任何配置/脏数据下
    // 都恰好有一个引擎处于激活态（否则 performSearch 会因拿不到 URL 而静默失灵）。
    //   1) 上次选择（仅当它仍匹配当前某个引擎，避免引擎被改名/删除后失效）
    //   2) 模板标记的默认引擎（带 bg-gradient-to-r class）
    //   3) 第一个引擎（配置里没有任何引擎标 default 时兜底）
    // 用数组 find 按 dataset 精确比较，而非把 localStorage 值拼进 CSS 选择器，
    // 防止脏数据（含引号等）触发 querySelector 抛异常、中断后续初始化。
    const savedEngine = localStorage.getItem('preferred_engine');
    const initialBtn =
        (savedEngine && [...engineBtns].find(b => b.dataset.url === savedEngine)) ||
        document.querySelector('.engine-btn.bg-gradient-to-r') ||
        engineBtns[0];
    if (initialBtn) {
        setActiveEngine(initialBtn);
    }

    // 搜索功能
    function performSearch() {
        const query = searchInput.value.trim();
        if (!query) {
            searchInput.focus();
            searchInput.classList.add('animate-pulse');
            setTimeout(() => searchInput.classList.remove('animate-pulse'), 500);
            return;
        }

        const currentEngineUrl = getActiveEngineUrl();
        if (!currentEngineUrl) {
            console.error('No search engine selected');
            return;
        }

        const searchUrl = currentEngineUrl + encodeURIComponent(query);
        window.open(searchUrl, '_blank');
    }

    searchBtn.addEventListener('click', performSearch);
    searchInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            performSearch();
        }
    });

    // 自动聚焦（Zen Mode 下不自动聚焦，让搜索框保持隐藏直到鼠标唤出）
    if (!document.body.classList.contains('zen-mode')) {
        setTimeout(() => searchInput.focus(), 100);
    }

    // 快捷键处理 - 使用更可靠的方式
    document.addEventListener('keydown', (e) => {
        // / 键聚焦搜索框
        if (e.key === '/' && document.activeElement !== searchInput) {
            e.preventDefault();
            e.stopImmediatePropagation();
            searchInput.focus();
            searchInput.select();
            return;
        }

        // Z 键切换 Zen Mode
        if (e.key === 'z' || e.key === 'Z') {
            if (document.activeElement !== searchInput) {
                e.preventDefault();
                document.body.classList.toggle('zen-mode');
                const isNowZenMode = document.body.classList.contains('zen-mode');
                localStorage.setItem('zen_mode', isNowZenMode);
                if (isNowZenMode) document.body.classList.remove('reveal-search');
                console.log('Zen mode (keyboard):', isNowZenMode ? 'ON' : 'OFF');
                return;
            }
        }

        // W 键切换壁纸清晰度
        if (e.key === 'w' || e.key === 'W') {
            if (document.activeElement !== searchInput) {
                e.preventDefault();
                document.body.classList.toggle('clear-wallpaper');
                const isNowClear = document.body.classList.contains('clear-wallpaper');
                localStorage.setItem('clear_wallpaper', isNowClear);
                console.log('Wallpaper (keyboard):', isNowClear ? 'CLEAR' : 'BLURRED');
                return;
            }
        }

        // Escape 取消聚焦
        if (e.key === 'Escape' && document.activeElement === searchInput) {
            searchInput.blur();
            return;
        }
    }, true); // 使用 capture 阶段确保优先处理

    // 定时更新壁纸
    setInterval(async () => {
        try {
            const response = await fetch('/api/wallpaper');
            if (response.ok) {
                const data = await response.json();
                const wallpaper = document.querySelector('.wallpaper');
                if (data.url && wallpaper) {
                    wallpaper.style.opacity = '0';
                    setTimeout(() => {
                        wallpaper.style.backgroundImage = `url('${data.url}')`;
                        wallpaper.style.opacity = '0.2';
                    }, 500);
                }
            }
        } catch (error) {
            console.error('Failed to refresh wallpaper:', error);
        }
    }, 60 * 60 * 1000);

    // 实时更新时间显示
    function updateTimezones() {
        const tzElements = document.querySelectorAll('#timezone-container > div[data-tz]');

        tzElements.forEach(el => {
            const tzName = el.dataset.tz;
            const timeEl = el.querySelector('.tz-time');
            if (!timeEl) return;

            let now;
            if (tzName === 'local' || tzName === 'Local') {
                now = new Date();
            } else {
                // 使用 Intl.DateTimeFormat 获取指定时区的时间
                try {
                    const options = {
                        timeZone: tzName,
                        hour12: false,
                        hour: '2-digit',
                        minute: '2-digit',
                        second: '2-digit'
                    };
                    const timeString = new Intl.DateTimeFormat('en-US', options).format(new Date());
                    timeEl.textContent = timeString;
                    return;
                } catch (e) {
                    // 如果时区无效，使用本地时间
                    now = new Date();
                }
            }

            // 格式化时间 HH:MM:SS
            const hours = String(now.getHours()).padStart(2, '0');
            const minutes = String(now.getMinutes()).padStart(2, '0');
            const seconds = String(now.getSeconds()).padStart(2, '0');
            timeEl.textContent = `${hours}:${minutes}:${seconds}`;
        });
    }

    // 每秒更新一次时间
    updateTimezones(); // 立即执行一次
    setInterval(updateTimezones, 1000);
});

#import <AppKit/AppKit.h>
#import <objc/runtime.h>

// 全局变量
static NSMutableArray *g_clickedButtons = nil;
static NSView *g_custom_toolbar = nil;
static NSMutableDictionary *g_buttons = nil;

// 初始化
__attribute__((constructor))
static void initNativeToolbar() {
    g_clickedButtons = [[NSMutableArray alloc] init];
    g_buttons = [[NSMutableDictionary alloc] init];
}

// 按钮点击回调
void onToolbarButtonClicked(id self, SEL _cmd, id sender) {
    NSButton *button = (NSButton *)sender;
    NSString *buttonId = button.accessibilityIdentifier;
    if (buttonId) {
        @synchronized(g_clickedButtons) {
            [g_clickedButtons addObject:buttonId];
        }
        NSLog(@"工具栏按钮点击：%@", buttonId);
    }
}

// 创建带文字的按钮
NSButton *createToolbarButton(NSString *identifier, NSString *emoji, NSString *text) {
    // 创建 attributed string，包含 emoji 和文字
    NSString *title = [NSString stringWithFormat:@"%@ %@", emoji, text];
    
    // 创建按钮
    NSButton *button = [[NSButton alloc] initWithFrame:NSZeroRect];
    button.bezelStyle = NSBezelStyleRounded;
    button.accessibilityIdentifier = identifier;
    button.bordered = YES;
    
    // 使用 attributed title
    NSDictionary *attributes = @{
        NSFontAttributeName: [NSFont systemFontOfSize:13]
    };
    NSAttributedString *attrTitle = [[NSAttributedString alloc] initWithString:title attributes:attributes];
    button.attributedTitle = attrTitle;
    
    // 动态添加点击处理方法
    static dispatch_once_t onceToken;
    static Class buttonDelegateClass = nil;
    dispatch_once(&onceToken, ^{
        buttonDelegateClass = objc_allocateClassPair([NSObject class], "ToolbarButtonDelegate", 0);
        class_addMethod(buttonDelegateClass, @selector(click:), (IMP)onToolbarButtonClicked, "v@:@");
        objc_registerClassPair(buttonDelegateClass);
    });
    
    static id sharedDelegate = nil;
    static dispatch_once_t delegateOnceToken;
    dispatch_once(&delegateOnceToken, ^{
        sharedDelegate = [[buttonDelegateClass alloc] init];
    });
    
    button.target = sharedDelegate;
    button.action = @selector(click:);
    
    // 存储按钮引用
    g_buttons[identifier] = button;
    
    NSLog(@"创建按钮：%@, title: %@", identifier, title);
    
    return button;
}

// 创建自定义工具栏
void create_native_toolbar(void *windowPtr) {
    NSWindow *window = (__bridge NSWindow *)windowPtr;
    
    dispatch_async(dispatch_get_main_queue(), ^{
        // 创建自定义工具栏视图
        CGFloat toolbarHeight = 40;
        NSRect toolbarRect = NSMakeRect(0, 0, 800, toolbarHeight);
        
        // 使用 NSStackView 来排列按钮
        NSStackView *stackView = [[NSStackView alloc] initWithFrame:toolbarRect];
        stackView.orientation = NSUserInterfaceLayoutOrientationHorizontal;
        stackView.spacing = 8;
        stackView.alignment = NSLayoutAttributeCenterY;
        stackView.edgeInsets = NSEdgeInsetsMake(5, 15, 5, 15);
        
        // 创建工作区按钮
        NSButton *workspaceBtn = createToolbarButton(@"workspace", @"💼", @"工作区");
        [stackView addArrangedSubview:workspaceBtn];
        
        // 创建插件按钮
        NSButton *pluginsBtn = createToolbarButton(@"plugins", @"🔌", @"插件");
        [stackView addArrangedSubview:pluginsBtn];
        
        // 创建深色按钮
        NSButton *themeBtn = createToolbarButton(@"theme", @"🌙", @"深色");
        [stackView addArrangedSubview:themeBtn];
        
        // 添加空白间隔
        NSView *spacer = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 20, 1)];
        [stackView addArrangedSubview:spacer];
        
        // 创建上一个按钮
        NSButton *prevBtn = createToolbarButton(@"prev", @"⬆️", @"上一个");
        [stackView addArrangedSubview:prevBtn];
        
        // 创建下一个按钮
        NSButton *nextBtn = createToolbarButton(@"next", @"⬇️", @"下一个");
        [stackView addArrangedSubview:nextBtn];
        
        // 创建刷新按钮
        NSButton *refreshBtn = createToolbarButton(@"refresh", @"🔄", @"刷新");
        [stackView addArrangedSubview:refreshBtn];
        
        // 添加到窗口的 content view
        [window.contentView addSubview:stackView];
        g_custom_toolbar = stackView;
        
        // 设置约束
        stackView.translatesAutoresizingMaskIntoConstraints = NO;
        [NSLayoutConstraint activateConstraints:@[
            [stackView.topAnchor constraintEqualToAnchor:window.contentView.topAnchor],
            [stackView.leadingAnchor constraintEqualToAnchor:window.contentView.leadingAnchor],
            [stackView.trailingAnchor constraintEqualToAnchor:window.contentView.trailingAnchor],
            [stackView.heightAnchor constraintEqualToConstant:toolbarHeight]
        ]];
        
        NSLog(@"原生工具栏已创建");
    });
}

// 获取点击的按钮
const char *get_clicked_toolbar_button() {
    @synchronized(g_clickedButtons) {
        if ([g_clickedButtons count] > 0) {
            NSString *buttonId = [g_clickedButtons firstObject];
            [g_clickedButtons removeObjectAtIndex:0];
            return strdup([buttonId UTF8String]);
        }
    }
    return NULL;
}

// 设置按钮启用状态
void set_toolbar_button_enabled(const char *buttonId, int enabled) {
    NSString *idStr = [NSString stringWithUTF8String:buttonId];
    NSButton *button = g_buttons[idStr];
    if (button) {
        button.enabled = (enabled != 0);
    }
}

// 设置按钮标题
void set_toolbar_button_title(const char *buttonId, const char *title) {
    NSString *idStr = [NSString stringWithUTF8String:buttonId];
    NSString *titleStr = [NSString stringWithUTF8String:title];
    NSButton *button = g_buttons[idStr];
    if (button) {
        NSDictionary *attributes = @{
            NSFontAttributeName: [NSFont systemFontOfSize:13]
        };
        NSAttributedString *attrTitle = [[NSAttributedString alloc] initWithString:titleStr attributes:attributes];
        button.attributedTitle = attrTitle;
    }
}

#import <AppKit/AppKit.h>

// MARK: - 面板管理器

@interface NativePanelManager : NSObject
@property (nonatomic, strong) NSMutableArray<NSPanel *> *panels;
+ (instancetype)sharedManager;
- (void)showPanelWithContent:(NSString *)content title:(NSString *)title buttonText:(NSString *)buttonText;
- (void)showAlertWithTitle:(NSString *)title message:(NSString *)message buttonText:(NSString *)buttonText;
@end

@implementation NativePanelManager

+ (instancetype)sharedManager {
    static NativePanelManager *instance = nil;
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        instance = [[NativePanelManager alloc] init];
        instance.panels = [NSMutableArray array];
    });
    return instance;
}

- (void)showPanelWithContent:(NSString *)content title:(NSString *)title buttonText:(NSString *)buttonText {
    dispatch_async(dispatch_get_main_queue(), ^{
        NSPanel *panel = [[NSPanel alloc] initWithContentRect:NSMakeRect(0, 0, 450, 300)
                                                    styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable
                                                      backing:NSBackingStoreBuffered
                                                        defer:NO];
        
        [panel setTitle:title];
        [panel center];
        
        // 创建内容视图
        NSView *contentView = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 450, 300)];
        [contentView setWantsLayer:YES];
        [contentView.layer setBackgroundColor:[NSColor windowBackgroundColor].CGColor];
        
        // 标题标签
        NSTextField *titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 220, 410, 30)];
        [titleLabel setStringValue:title];
        [titleLabel setFont:[NSFont systemFontOfSize:18 weight:NSFontWeightBold]];
        [titleLabel setBezeled:NO];
        [titleLabel setDrawsBackground:NO];
        [titleLabel setEditable:NO];
        [titleLabel setAlignment:NSTextAlignmentCenter];
        [contentView addSubview:titleLabel];
        
        // 内容标签
        NSTextField *contentLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 100, 410, 100)];
        [contentLabel setStringValue:content];
        [contentLabel setFont:[NSFont systemFontOfSize:14]];
        [contentLabel setTextColor:[NSColor secondaryLabelColor]];
        [contentLabel setBezeled:NO];
        [contentLabel setDrawsBackground:NO];
        [contentLabel setEditable:NO];
        [contentLabel setAlignment:NSTextAlignmentCenter];
        [contentLabel setLineBreakMode:NSLineBreakByWordWrapping];
        [contentView addSubview:contentLabel];
        
        // 按钮
        NSButton *button = [[NSButton alloc] initWithFrame:NSMakeRect(150, 30, 150, 40)];
        [button setTitle:buttonText];
        [button setBezelStyle:NSBezelStyleRounded];
        [button setTarget:self];
        [button setAction:@selector(panelButtonClicked:)];
        [button setTag:[self.panels count]];
        [contentView addSubview:button];
        
        [panel setContentView:contentView];
        [panel makeKeyAndOrderFront:nil];
        
        [self.panels addObject:panel];
    });
}

- (void)panelButtonClicked:(NSButton *)sender {
    if ([self.panels count] > 0) {
        NSPanel *panel = [self.panels firstObject];
        [panel close];
        [self.panels removeObject:panel];
    }
    NSLog(@"原生面板按钮被点击");
}

- (void)showAlertWithTitle:(NSString *)title message:(NSString *)message buttonText:(NSString *)buttonText {
    dispatch_async(dispatch_get_main_queue(), ^{
        NSAlert *alert = [[NSAlert alloc] init];
        [alert setMessageText:title];
        [alert setInformativeText:message];
        [alert addButtonWithTitle:buttonText];
        [alert setAlertStyle:NSAlertStyleInformational];
        [alert runModal];
        NSLog(@"警告框已确认");
    });
}

@end

// MARK: - C 接口（供 Rust 调用）

void swift_show_panel(const char *content, const char *title, const char *buttonText) {
    NSString *contentStr = [NSString stringWithUTF8String:content];
    NSString *titleStr = [NSString stringWithUTF8String:title];
    NSString *buttonStr = [NSString stringWithUTF8String:buttonText];
    
    [[NativePanelManager sharedManager] showPanelWithContent:contentStr title:titleStr buttonText:buttonStr];
}

void swift_show_alert(const char *title, const char *message, const char *buttonText) {
    NSString *titleStr = [NSString stringWithUTF8String:title];
    NSString *messageStr = [NSString stringWithUTF8String:message];
    NSString *buttonStr = [NSString stringWithUTF8String:buttonText];
    
    [[NativePanelManager sharedManager] showAlertWithTitle:titleStr message:messageStr buttonText:buttonStr];
}

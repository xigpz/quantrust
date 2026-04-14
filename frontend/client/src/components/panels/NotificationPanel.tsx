import { useState, useEffect } from 'react';
import { useNotificationChannels, updateNotificationChannels, sendNotification, useNotificationHistory, type NotificationChannels as NotificationChannelsType } from '@/hooks/useMarketData';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Loader2, Bell, Send, History } from 'lucide-react';
import { toast } from 'sonner';

export default function NotificationPanel() {
  const { data: channels, loading, refetch } = useNotificationChannels();
  const { data: history } = useNotificationHistory(20);
  const [form, setForm] = useState<Partial<NotificationChannelsType>>({});
  const [saving, setSaving] = useState(false);
  const [sendingNotif, setSendingNotif] = useState(false);
  const [notifTitle, setNotifTitle] = useState('');
  const [notifContent, setNotifContent] = useState('');

  useEffect(() => {
    if (channels) {
      setForm(channels);
    }
  }, [channels]);

  const handleSave = async () => {
    setSaving(true);
    try {
      const res = await updateNotificationChannels(form);
      if (res.success) {
        toast.success('保存成功', { description: '通知渠道配置已更新' });
        refetch();
      } else {
        toast.error('保存失败', { description: res.message });
      }
    } catch (e) {
      toast.error('保存失败');
    } finally {
      setSaving(false);
    }
  };

  const handleTestSend = async () => {
    if (!notifTitle || !notifContent) return;
    setSendingNotif(true);
    try {
      const res = await sendNotification({
        title: notifTitle,
        content: notifContent,
        notification_type: 'system'
      });
      if (res.success) {
        toast.success('发送成功', { description: '测试通知已发送' });
      } else {
        toast.error('发送失败', { description: res.message });
      }
    } catch (e) {
      toast.error('发送失败');
    } finally {
      setSendingNotif(false);
    }
  };

  if (loading) {
    return <div className="p-4"><Loader2 className="h-8 w-8 animate-spin" /></div>;
  }

  return (
    <div className="p-4 space-y-4 overflow-auto h-full">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold flex items-center gap-2">
          <Bell className="h-6 w-6" />
          通知设置
        </h2>
      </div>

      {/* 通知渠道配置 */}
      <Card>
        <CardHeader>
          <CardTitle>通知渠道配置</CardTitle>
          <CardDescription>配置后可用于接收交易信号、风险告警等通知</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>飞书机器人 Webhook</Label>
              <Input
                type="password"
                placeholder="https://open.feishu.cn/..."
                value={form.feishu_webhook || ''}
                onChange={(e) => setForm({ ...form, feishu_webhook: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>企业微信 Webhook</Label>
              <Input
                type="password"
                placeholder="https://qyapi.weixin.qq.com/..."
                value={form.wecom_webhook || ''}
                onChange={(e) => setForm({ ...form, wecom_webhook: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>Telegram Bot Token</Label>
              <Input
                type="password"
                placeholder="123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
                value={form.telegram_bot_token || ''}
                onChange={(e) => setForm({ ...form, telegram_bot_token: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>Telegram Chat ID</Label>
              <Input
                placeholder="123456789"
                value={form.telegram_chat_id || ''}
                onChange={(e) => setForm({ ...form, telegram_chat_id: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>Discord Webhook</Label>
              <Input
                type="password"
                placeholder="https://discord.com/api/webhooks/..."
                value={form.discord_webhook || ''}
                onChange={(e) => setForm({ ...form, discord_webhook: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>钉钉机器人 Webhook</Label>
              <Input
                type="password"
                placeholder="https://oapi.dingtalk.com/..."
                value={form.dingtalk_webhook || ''}
                onChange={(e) => setForm({ ...form, dingtalk_webhook: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>PushPlus Token</Label>
              <Input
                type="password"
                placeholder="pushplus token"
                value={form.pushplus_token || ''}
                onChange={(e) => setForm({ ...form, pushplus_token: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>ServerChan Key</Label>
              <Input
                type="password"
                placeholder="serverchan key"
                value={form.serverchan_key || ''}
                onChange={(e) => setForm({ ...form, serverchan_key: e.target.value })}
              />
            </div>
          </div>
          <Button onClick={handleSave} disabled={saving}>
            {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            保存配置
          </Button>
        </CardContent>
      </Card>

      {/* 测试发送 */}
      <Card>
        <CardHeader>
          <CardTitle>测试发送</CardTitle>
          <CardDescription>发送测试通知验证配置是否正确</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div>
            <Label>标题</Label>
            <Input
              placeholder="测试通知"
              value={notifTitle}
              onChange={(e) => setNotifTitle(e.target.value)}
            />
          </div>
          <div>
            <Label>内容</Label>
            <Input
              placeholder="这是一条测试消息"
              value={notifContent}
              onChange={(e) => setNotifContent(e.target.value)}
            />
          </div>
          <Button onClick={handleTestSend} disabled={sendingNotif || !notifTitle || !notifContent}>
            {sendingNotif && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            <Send className="mr-2 h-4 w-4" />
            发送测试
          </Button>
        </CardContent>
      </Card>

      {/* 通知历史 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <History className="h-5 w-5" />
            通知历史
          </CardTitle>
        </CardHeader>
        <CardContent>
          {history && history.length > 0 ? (
            <div className="space-y-2 max-h-60 overflow-y-auto">
              {history.map((notif, i) => (
                <div key={i} className="border p-2 rounded text-sm">
                  <div className="flex justify-between">
                    <span className="font-medium">{notif.title}</span>
                    <span className="text-xs text-muted-foreground">
                      {new Date(notif.timestamp * 1000).toLocaleString()}
                    </span>
                  </div>
                  <div className="text-muted-foreground mt-1">{notif.content}</div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-muted-foreground text-sm">暂无通知记录</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

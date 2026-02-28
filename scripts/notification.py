#!/usr/bin/env python3
"""
通知服务 - 飞书/邮件/钉钉推送
"""

import requests
import json
import smtplib
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from datetime import datetime
from typing import List, Optional

class FeishuNotifier:
    """飞书机器人通知"""
    
    def __init__(self, webhook_url: str):
        self.webhook_url = webhook_url
    
    def send(self, title: str, content: str) -> bool:
        """发送消息"""
        payload = {
            "msg_type": "text",
            "content": {
                "text": f"【{title}】\n{content}"
            }
        }
        
        try:
            resp = requests.post(self.webhook_url, json=payload, timeout=10)
            return resp.status_code == 200
        except Exception as e:
            print(f"飞书发送失败: {e}")
            return False


class EmailNotifier:
    """邮件通知"""
    
    def __init__(self, smtp_host: str, smtp_port: int, username: str, password: str, 
                 from_addr: str, to_addrs: List[str]):
        self.smtp_host = smtp_host
        self.smtp_port = smtp_port
        self.username = username
        self.password = password
        self.from_addr = from_addr
        self.to_addrs = to_addrs
    
    def send(self, title: str, content: str) -> bool:
        """发送邮件"""
        msg = MIMEMultipart('alternative')
        msg['Subject'] = title
        msg['From'] = self.from_addr
        msg['To'] = ','.join(self.to_addrs)
        
        # HTML 内容
        html = f"""
        <html>
        <body>
            <h2>{title}</h2>
            <pre>{content}</pre>
            <hr>
            <p style="color: gray; font-size: 12px;">
                发送时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
            </p>
        </body>
        </html>
        """
        
        msg.attach(MIMEText(content, 'plain'))
        msg.attach(MIMEText(html, 'html'))
        
        try:
            server = smtplib.SMTP(self.smtp_host, self.smtp_port)
            server.starttls()
            server.login(self.username, self.password)
            server.send_message(msg)
            server.quit()
            return True
        except Exception as e:
            print(f"邮件发送失败: {e}")
            return False


class DingTalkNotifier:
    """钉钉机器人通知"""
    
    def __init__(self, webhook_url: str, secret: str = None):
        self.webhook_url = webhook_url
        self.secret = secret
    
    def send(self, title: str, content: str) -> bool:
        """发送消息"""
        import time
        import hmac
        import hashlib
        import base64
        import urllib.parse
        
        # 如果有密钥，需要签名
        timestamp = str(round(time.time() * 1000))
        if self.secret:
            secret_enc = self.secret.encode('utf-8')
            string_to_sign = f'{timestamp}\n{self.secret}'
            string_to_sign_enc = string_to_sign.encode('utf-8')
            hmac_code = hmac.new(secret_enc, string_to_sign_enc, digestmod=hashlib.sha256).digest()
            sign = urllib.parse.quote_plus(base64.b64encode(hmac_code))
            
            url = f"{self.webhook_url}&timestamp={timestamp}&sign={sign}"
        else:
            url = self.webhook_url
        
        payload = {
            "msgtype": "text",
            "text": {
                "content": f"【{title}】\n{content}"
            }
        }
        
        try:
            resp = requests.post(url, json=payload, timeout=10)
            return resp.status_code == 200
        except Exception as e:
            print(f"钉钉发送失败: {e}")
            return False


class NotificationManager:
    """通知管理器"""
    
    def __init__(self):
        self.feishu = None
        self.email = None
        self.dingtalk = None
    
    def set_feishu(self, webhook_url: str):
        self.feishu = FeishuNotifier(webhook_url)
    
    def set_email(self, smtp_host: str, smtp_port: int, username: str, 
                  password: str, from_addr: str, to_addrs: List[str]):
        self.email = EmailNotifier(smtp_host, smtp_port, username, password, from_addr, to_addrs)
    
    def set_dingtalk(self, webhook_url: str, secret: str = None):
        self.dingtalk = DingTalkNotifier(webhook_url, secret)
    
    def send_trade_signal(self, symbol: str, signal: str, price: float):
        """发送交易信号"""
        title = f"交易信号 - {symbol}"
        content = f"信号: {signal}\n价格: {price:.2f}\n时间: {datetime.now().strftime('%H:%M:%S')}"
        self._send_all(title, content)
    
    def send_risk_alert(self, alert_type: str, message: str):
        """发送风险告警"""
        title = f"风险告警 - {alert_type}"
        content = f"{message}\n时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}"
        self._send_all(title, content)
    
    def send_flow_alert(self, symbol: str, inflow: float):
        """发送资金异动"""
        direction = "流入" if inflow > 0 else "流出"
        title = f"资金异动 - {symbol}"
        content = f"主力{direction}: {abs(inflow):.1f}万\n时间: {datetime.now().strftime('%H:%M:%S')}"
        self._send_all(title, content)
    
    def send_report(self, report_type: str, content: str):
        """发送报告"""
        title = f"{report_type}报告"
        self._send_all(title, content)
    
    def _send_all(self, title: str, content: str):
        """发送给所有渠道"""
        if self.feishu:
            self.feishu.send(title, content)
        if self.email:
            self.email.send(title, content)
        if self.dingtalk:
            self.dingtalk.send(title, content)


# 示例
if __name__ == "__main__":
    # 初始化
    notifier = NotificationManager()
    
    # 配置飞书 (替换为你的 webhook)
    # notifier.set_feishu("https://open.feishu.cn/open-apis/bot/v2/hook/xxx")
    
    # 配置邮件
    # notifier.set_email("smtp.qq.com", 587, "user@qq.com", "password", "user@qq.com", ["to@example.com"])
    
    # 测试
    notifier.send_trade_signal("600519", "买入", 1800.5)
    notifier.send_risk_alert("止损", "触发5%止损线")
    notifier.send_flow_alert("000858", 15000)

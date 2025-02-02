import sys
import smtplib
import ssl

def send_test_email(outlook_username, outlook_password, from_address, to_address, smtp_server = "smtp.office365.com", body=""):
    port = 587
    subject = "Test Email from Python (Outlook SMTP)"

    message = f"""From: {from_address}
    To: {to_address}
    Subject: {subject}

    {body}
    """

    context = ssl.create_default_context()
    with smtplib.SMTP(smtp_server, port) as server:
        server.set_debuglevel(1)  
        server.ehlo()
        server.starttls(context=context)
        server.ehlo()
        server.login(outlook_username, outlook_password)
        server.sendmail(from_address, to_address, message)


if __name__ == "__main__":
    if len(sys.argv) < 6:
        print("Usage: python -c <this code> <username> <password> <from> <to> <server>")
        sys.exit(1)
    send_test_email(sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], smtp_server=sys.argv[5], body=sys.argv[6])

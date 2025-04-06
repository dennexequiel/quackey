# Quackey Usage Guide

This guide provides detailed instructions on how to use Quackey.

## Getting Started

### First Run

When you run Quackey for the first time, you'll be prompted to configure the application:

1. Choose between default configuration or custom storage location
2. The application will create necessary directories and files
3. You'll see a welcome message confirming successful setup

### Main Menu Navigation

The main menu offers four options:

- 🔢 Generate TOTP
- 📂 Manage Accounts
- ⚙️ Configure Settings
- 🦆 Exit

Use arrow keys to navigate and Enter to select an option.

## Account Management

### Adding a New Account

1. Select "📂 Manage Accounts" from the main menu
2. Choose "📄 Add new account"
3. Enter the required information:
   - Account name (e.g., email or username)
   - Issuer (optional, e.g., Google, GitHub)
   - Secret key (from your service provider)
   - TOTP parameters:
     - Digits (6, 7, or 8)
     - Period (30, 60, or 90 seconds)
     - Algorithm (SHA1, SHA256, SHA512)

### Editing an Account

1. Select "📂 Manage Accounts"
2. Choose "📝 Edit account"
3. Select the account to edit
4. Modify the desired fields
5. Press Enter to save changes

### Deleting an Account

1. Select "📂 Manage Accounts"
2. Choose "🗑️ Delete account"
3. Select the account to delete
4. Confirm deletion

### Viewing Accounts

1. Select "📂 Manage Accounts"
2. Choose "👀 View saved accounts"
3. A table will display all your accounts with their details

## Generating TOTP Codes

1. Select "🔢 Generate TOTP" from the main menu
2. Choose an account from the list
3. The application will display:
   - Current TOTP code
   - Time remaining until code refresh
   - Account details

## Configuration

### Changing Storage Location

1. Select "⚙️ Configure Settings"
2. Choose to modify storage location
3. Enter new path
4. Confirm changes

### Viewing Logs

Logs are stored in `totp_app.log` in your application directory. They contain:
- Application startup/shutdown events
- Account modifications
- TOTP generation attempts
- Error messages

## Tips and Best Practices

1. **Secret Key Management**
   - Keep your secret keys secure
   - Don't share them with anyone
   - Store backup copies safely

2. **Account Organization**
   - Use descriptive account names
   - Include issuer names when available
   - Group related accounts with similar names

3. **Security**
   - Regularly backup your accounts.json file
   - Don't store the application on shared systems

## Troubleshooting

### Common Issues

1. **Invalid Secret Key**
   - Ensure the key is correctly copied
   - Check for extra spaces
   - Verify the key length

2. **Configuration Errors**
   - Check file permissions
   - Verify path exists
   - Ensure write access

3. **TOTP Generation Issues**
   - Verify system time is correct
   - Check algorithm compatibility
   - Confirm period settings

### Getting Help

If you encounter issues not covered here:
1. Check the logs in `totp_app.log`
2. Submit an issue on GitHub
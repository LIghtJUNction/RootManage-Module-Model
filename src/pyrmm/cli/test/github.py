import click
import os

@click.command()
@click.option("--token", envvar="GITHUB_ACCESS_TOKEN", help="GitHub访问令牌，默认从环境变量GITHUB_ACCESS_TOKEN获取")
@click.option("--repo", "-r", help="检测对特定仓库的访问权限 (格式: username/repo)")
@click.option("--verbose", "-v", is_flag=True, help="显示详细信息")
def github(token: str | None, repo: str | None, verbose: bool) -> None:
    """检测GitHub token是否有效，检测权限"""
    from pyrmm.usr.lib.git import RmmGit
    
    try:
        # 检查GitHub库是否可用
        try:
            from github import Github
            from github.GithubException import GithubException
        except ImportError:
            click.echo("❌ PyGithub库未安装，无法使用GitHub API功能")
            click.echo("💡 安装命令: pip install PyGithub")
            return
          # 获取token
        if not token:
            token = os.getenv('GITHUB_ACCESS_TOKEN')
            if not token:
                click.echo("❌ 未提供GitHub访问令牌")
                click.echo("💡 请通过以下方式之一提供token:")
                click.echo("   1. 使用 --token 参数: rmm test github --token YOUR_TOKEN")
                click.echo("   2. 设置环境变量: $env:GITHUB_ACCESS_TOKEN='YOUR_TOKEN' (PowerShell)")
                click.echo("   3. 或在环境变量中设置 GITHUB_ACCESS_TOKEN")
                click.echo("🔗 创建token: https://github.com/settings/tokens/new")
                click.echo("\n🔧 当前环境变量状态:")
                env_token = os.getenv('GITHUB_ACCESS_TOKEN')
                if env_token:
                    click.echo(f"   GITHUB_ACCESS_TOKEN: 已设置 (长度: {len(env_token)})")
                else:
                    click.echo("   GITHUB_ACCESS_TOKEN: 未设置")
                return
        
        # 验证token格式
        if not token.startswith(('ghp_', 'github_pat_')):
            click.echo("⚠️  警告: Token格式可能不正确")
            click.echo("💡 GitHub个人访问令牌通常以 'ghp_' 或 'github_pat_' 开头")
        
        click.echo("🔑 正在验证GitHub token...")
        
        # 显示token信息（脱敏）
        if verbose:
            token_preview = f"{token[:7]}...{token[-4:]}" if len(token) > 11 else f"{token[:4]}***"
            click.echo(f"🔍 Token: {token_preview}")
        
        # 验证token基本有效性
        try:
            g = Github(token)
            user = g.get_user()
            
            # 尝试获取用户名来验证token是否真正有效
            username = user.login
            
            click.echo("✅ Token验证成功!")
            click.echo(f"👤 用户名: {username}")
            
            # 安全地获取用户信息
            try:
                email = user.email if hasattr(user, 'email') else None
                click.echo(f"📧 邮箱: {email or '未公开'}")
            except:
                click.echo("📧 邮箱: 无法获取")
            
            try:
                company = user.company if hasattr(user, 'company') else None
                click.echo(f"🏢 公司: {company or '未设置'}")
            except:
                click.echo("🏢 公司: 无法获取")
            
            try:
                location = user.location if hasattr(user, 'location') else None
                click.echo(f"📍 位置: {location or '未设置'}")
            except:
                click.echo("📍 位置: 无法获取")
            
            # 检查API限制
            try:
                rate_limit = g.get_rate_limit()
                click.echo(f"⏱️  API限制: {rate_limit.core.remaining}/{rate_limit.core.limit}")
            except Exception as e:
                click.echo(f"⏱️  API限制: 无法获取 ({e})")
            
            # 检查token权限
            click.echo("\n🔐 权限检测:")
            
            # 检查基本权限
            permissions: list[str] = []

            # 检查是否可以获取用户邮箱（需要user:email权限）
            try:
                # 尝试一个需要user:email权限的操作
                user_data = g.get_user()
                if hasattr(user_data, 'email') and user_data.email:
                    permissions.append("✅ user:email (读取邮箱权限)")
                else:
                    permissions.append("⚠️  user:email (可能无权限或邮箱未公开)")
            except GithubException as e:
                if e.status == 403:
                    permissions.append("❌ user:email (权限不足)")
                else:
                    permissions.append(f"❌ user:email (检测失败: {e.status})")
            except Exception:
                permissions.append("❌ user:email (检测失败)")
            
            # 检查仓库权限（尝试获取私有仓库）
            try:
                # 尝试获取用户的仓库列表
                repo_count = 0
                for _ in g.get_user().get_repos():
                    repo_count += 1
                    if repo_count >= 1:  # 只检查第一个仓库
                        break
                permissions.append("✅ repo (仓库访问权限)")
            except GithubException as e:
                if e.status == 403:
                    permissions.append("❌ repo (权限不足)")
                else:
                    permissions.append(f"❌ repo (检测失败: {e.status})")
            except Exception:
                permissions.append("❌ repo (检测失败)")
            
            # 检查组织权限
            try:
                org_count = 0
                for _ in g.get_user().get_orgs():
                    org_count += 1
                    if org_count >= 1:  # 只检查是否能获取组织
                        break
                permissions.append(f"✅ read:org (组织权限)")
            except GithubException as e:
                if e.status == 403:
                    permissions.append("❌ read:org (权限不足)")
                else:
                    permissions.append(f"❌ read:org (检测失败: {e.status})")
            except Exception:
                permissions.append("❌ read:org (检测失败)")
            
            for perm in permissions:
                click.echo(f"  {perm}")
            
            # 如果指定了仓库，检查对该仓库的访问权限
            if repo:
                click.echo(f"\n📦 检测仓库访问权限: {repo}")
                try:
                    username, repo_name = repo.split('/', 1)
                except ValueError:
                    click.echo("❌ 仓库格式错误，应为: username/repository")
                    return
                
                # 检查仓库是否存在并可访问
                if RmmGit.check_repo_exists(username, repo_name, token):
                    click.echo("✅ 仓库访问成功")
                    
                    # 获取仓库详细信息
                    try:
                        repo_obj = g.get_repo(repo)
                        click.echo(f"📋 仓库名称: {repo_obj.full_name}")
                        
                        try:
                            description = repo_obj.description if hasattr(repo_obj, 'description') else None
                            click.echo(f"📝 描述: {description or '无描述'}")
                        except:
                            click.echo("📝 描述: 无法获取")
                        
                        try:
                            is_private = repo_obj.private if hasattr(repo_obj, 'private') else False
                            click.echo(f"🔒 私有仓库: {'是' if is_private else '否'}")
                        except:
                            click.echo("🔒 私有仓库: 无法确定")
                        
                        try:
                            stars = repo_obj.stargazers_count if hasattr(repo_obj, 'stargazers_count') else 0
                            click.echo(f"⭐ Stars: {stars}")
                        except:
                            click.echo("⭐ Stars: 无法获取")
                        
                        try:
                            forks = repo_obj.forks_count if hasattr(repo_obj, 'forks_count') else 0
                            click.echo(f"🍴 Forks: {forks}")
                        except:
                            click.echo("🍴 Forks: 无法获取")
                          # 检查是否可以读取releases
                        try:
                            releases = list(repo_obj.get_releases())
                            if len(releases) > 0:
                                click.echo(f"✅ 可以读取Releases ({len(releases)} 个)")
                            else:
                                click.echo("⚠️  该仓库没有Releases")
                        except GithubException as e:
                            click.echo(f"❌ 无法读取Releases: HTTP {e.status}")
                        except Exception as e:
                            click.echo(f"❌ 无法读取Releases: {e}")
                          # 检查是否可以读取仓库内容
                        try:
                            contents = repo_obj.get_contents("/")
                            # 处理返回值可能是单个文件或文件列表的情况
                            if isinstance(contents, list):
                                if len(contents) > 0:
                                    click.echo("✅ 可以读取仓库内容")
                                else:
                                    click.echo("⚠️  仓库内容为空")
                            else:
                                # 单个文件的情况
                                click.echo("✅ 可以读取仓库内容")
                        except GithubException as e:
                            click.echo(f"❌ 无法读取仓库内容: HTTP {e.status}")
                        except Exception as e:
                            click.echo(f"❌ 无法读取仓库内容: {e}")
                            
                    except GithubException as e:
                        click.echo(f"❌ 获取仓库详细信息失败: HTTP {e.status}")
                    except Exception as e:
                        click.echo(f"❌ 获取仓库详细信息失败: {e}")
                else:
                    click.echo("❌ 无法访问仓库")
                    click.echo("💡 可能的原因:")
                    click.echo("   1. 仓库不存在")
                    click.echo("   2. 仓库为私有且token无访问权限")
                    click.echo("   3. token权限不足")
            
            # 显示建议的token权限
            click.echo(f"\n💡 建议的token权限配置:")
            click.echo("   ✅ repo (完整仓库权限)")
            click.echo("   ✅ user:email (读取邮箱)")
            click.echo("   ✅ read:org (读取组织信息)")
            click.echo("   ✅ workflow (GitHub Actions权限，如需要)")
            
        except GithubException as e:
            click.echo(f"❌ Token验证失败: {e}")
            if e.status == 401:
                click.echo("🔍 错误分析: 认证失败")
                click.echo("💡 可能的解决方案:")
                click.echo("   1. 检查token是否正确")
                click.echo("   2. 检查token是否已过期")
                click.echo("   3. 重新生成token")
            elif e.status == 403:
                click.echo("🔍 错误分析: 权限不足或API限制")
                click.echo("💡 可能的解决方案:")
                click.echo("   1. 检查token权限范围")
                click.echo("   2. 等待API限制重置")
                click.echo("   3. 使用具有更高权限的token")
            else:
                click.echo(f"🔍 HTTP状态码: {e.status}")
            
            click.echo("🔗 管理tokens: https://github.com/settings/tokens")
            
        except Exception as e:
            click.echo(f"❌ 验证过程中出错: {e}")
            if verbose:
                import traceback
                traceback.print_exc()
    
    except Exception as e:
        click.echo(f"❌ 执行过程中出错: {e}")
        if verbose:
            import traceback
            traceback.print_exc()
import json
import matplotlib.pyplot as plt
import numpy as np
import click
from collections import namedtuple

Calc = namedtuple('Calc', [
    'wins',
    'losses',
    'saves',
    'goals',
    'assists',
    'shots',
    'lose_score',
    'win_score',
    'goal_diff',
    'time_diff',
    'name'
])

@click.command()
@click.argument('files', nargs=-1, type=click.File('rb'))
def run_analysis(files):
    data = [json.load(f) for f in files]
    wins = 0
    losses = 0
    saves = 0
    goals = 0
    assists = 0
    shots = 0
    lose_score = []
    win_score = []
    goal_diff = []
    time_diff = np.array([])

    for game in data:
        props = game['properties']
        name = props['PlayerName']
        fps = props['RecordFPS']
        player = find_player_team(props['PlayerStats'], name)
        team = player.get('Team')
        team0_score = props.get('Team0Score', 0)
        team1_score = props.get('Team1Score', 0)
        frames = [x['frame'] / fps for x in props['Goals']]
        time_diff = np.append(time_diff, np.diff([0] + frames))
        if team == 0 and team0_score > team1_score:
            wins = wins + 1
            win_score.append(player.get('Score',0))
        elif team == 1 and team1_score > team0_score:
            wins = wins + 1
            win_score.append(player.get('Score',0))
        else:
            losses = losses + 1
            lose_score.append(player.get('Score',0))
        goal_diff.append(team0_score - team1_score if team == 0 else team1_score - team0_score)
        saves = saves + player.get('Saves', 0)
        goals = goals + player.get('Goals', 0)
        assists = assists + player.get('Assists', 0)
        shots = shots + player.get('Shots', 0)
    c = Calc(wins, losses, saves, goals, assists, shots, lose_score, win_score, goal_diff, time_diff, name)
    graph(c)
    input()


def find_player_team(player_stats, name):
    for stat in player_stats:
        if stat['Name'] == name:
            return stat
    raise Exception("Did not see player name")



def autolabel(rects, ax):
    for rect in rects:
        h = rect.get_height()
        ax.text(rect.get_x()+rect.get_width()/2., 1.05*h + .1, '%d'%int(h),
                ha='center', va='bottom', size='xx-large')

def graph(calc):
    fig = plt.figure()
    with plt.xkcd():
        ind = np.arange(2)  # the x locations for the groups
        width = 0.50       # the width of the bars
        ax = fig.add_subplot(111)
        barlist = ax.bar(ind, [calc.wins, calc.losses], width, align = 'center')
        ax.set_ylim([0, max(calc.wins, calc.losses) * 1.2])
        ax.set_xticks(ind)
        ax.set_xticklabels(('Wins', 'Losses'), fontdict={ 'fontsize': 'xx-large' })
        barlist[0].set_facecolor('#7B9F35')
        barlist[1].set_facecolor('#AA3939')
        plt.title("Wins vs. losses", fontdict={ 'fontsize': 'xx-large' }, y = 1.05)
        plt.ylabel("Games", fontdict={ 'fontsize': 'xx-large' })
        autolabel(barlist, ax)
        fig.show()

    fig = plt.figure()
    with plt.xkcd():
        ind = np.arange(4)  # the x locations for the groups
        width = 0.50       # the width of the bars
        ax = fig.add_subplot(111)
        barlist = ax.bar(ind, [calc.saves, calc.goals, calc.shots, calc.assists], width, align = 'center', color="#226666")
        ax.set_ylim([0, max(calc.saves, calc.goals, calc.shots, calc.assists) * 1.2])
        ax.set_xticks(ind)
        ax.set_xticklabels(('Saves', 'Goals', 'Shots', 'Assists'), fontdict={ 'fontsize': 'large' })
        plt.title("Stats Breakdown", fontdict={ 'fontsize': 'xx-large' }, y=1.05)
        plt.ylabel("Count", fontdict={ 'fontsize': 'xx-large' })
        autolabel(barlist, ax)
        fig.show()

    fig = plt.figure()
    with plt.xkcd():
        ax = fig.add_subplot(111)
        plt.title('Player\'s Score Distribution: \nWins vs. Losses', fontdict={ 'fontsize': 'xx-large' }, y=1.05)
        bplot = ax.boxplot([calc.win_score, calc.lose_score], vert=True, patch_artist=True)
        bplot['boxes'][0].set_facecolor('#7B9F35')
        bplot['boxes'][1].set_facecolor('#AA3939')
        plt.ylabel("Score", fontdict={ 'fontsize': 'xx-large' })
        ax.set_ylim([0, max(max(*calc.win_score), max(*calc.lose_score)) * 1.2])
        ax.set_xticklabels(('Wins', 'Losses'), fontdict={ 'fontsize': 'xx-large' })
        fig.subplots_adjust(top=0.8)
        fig.show()

    fig = plt.figure()
    with plt.xkcd():
        ax = fig.add_subplot(111)
        plt.title('Goal Difference', fontdict={ 'fontsize': 'xx-large' }, y=1.05)
        plt.ylabel('Frequency')
        plt.xlabel('Goals')
        bplot = ax.hist(calc.goal_diff, color="#226666")
        fig.show()

    fig = plt.figure()
    with plt.xkcd():
        ax = fig.add_subplot(111)
        plt.title('Time Between Goals', fontdict={ 'fontsize': 'xx-large' }, y=1.05)
        plt.ylabel('Frequency')
        plt.xlabel('Seconds')
        bplot = ax.hist(calc.time_diff, color="#226666")
        fig.show()

if __name__ == '__main__':
    run_analysis()

